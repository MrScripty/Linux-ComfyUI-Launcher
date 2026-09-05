//! HTTP server implementation using Axum.

use crate::catalog_projection::CatalogProjection;
use crate::handlers::{
    handle_health, handle_model_download_update_events, handle_model_library_update_events,
    handle_rpc, handle_status_telemetry_update_events,
};
#[cfg(feature = "inference-plugins")]
use crate::handlers::{
    handle_openai_models, handle_openai_proxy, handle_runtime_profile_update_events,
    handle_serving_status_update_events,
};
#[cfg(feature = "inference-plugins")]
use crate::provider_clients::{LlamaCppRouterClient, OllamaClientFactory};
use axum::{
    extract::DefaultBodyLimit,
    http::{header, HeaderValue, Method},
    routing::{get, post},
    Router,
};
#[cfg(feature = "inference-plugins")]
use pumas_app_manager::{SizeCalculator, VersionManager};
use pumas_library::PumasApi;
#[cfg(feature = "inference-plugins")]
use pumas_library::{
    models::RuntimeEndpointUrl, OnnxEmbeddingBackendKind, OnnxSessionManager, PluginLoader,
    ProviderRegistry,
};
#[cfg(feature = "inference-plugins")]
use std::collections::HashMap;
use std::future::IntoFuture;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::sync::Arc;
#[cfg(feature = "inference-plugins")]
use std::time::Duration;
use tokio::sync::oneshot;
#[cfg(feature = "inference-plugins")]
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use tower::limit::ConcurrencyLimitLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::info;

const MAX_IN_FLIGHT_RPC_REQUESTS: usize = 64;
const MAX_REQUEST_BODY_BYTES: usize = 32 * 1024 * 1024;
#[cfg(feature = "inference-plugins")]
const GATEWAY_PROXY_TIMEOUT: Duration = Duration::from_secs(120);
#[cfg(feature = "inference-plugins")]
const PROVIDER_HTTP_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
#[cfg(feature = "inference-plugins")]
const ONNX_MAX_CONCURRENT_OPERATIONS: usize = 4;

/// A validated desktop RPC bind host.
///
/// The private field makes a non-loopback server configuration
/// unrepresentable after CLI admission.
#[derive(Clone, Copy)]
pub(crate) struct LoopbackHost(IpAddr);

impl LoopbackHost {
    pub(crate) fn parse(host: &str) -> anyhow::Result<Self> {
        let address: IpAddr = host
            .parse()
            .map_err(|_| anyhow::anyhow!("RPC host must be a loopback IP address"))?;
        if !address.is_loopback() {
            return Err(anyhow::anyhow!(
                "RPC host must be a loopback IP address; remote access is not supported"
            ));
        }
        Ok(Self(address))
    }

    const fn socket_addr(self, port: u16) -> SocketAddr {
        SocketAddr::new(self.0, port)
    }
}

/// Application state shared across handlers.
pub struct AppState {
    pub(crate) catalog_projection: CatalogProjection,
    /// Core API (model library, system utilities)
    pub api: PumasApi,
    /// Version managers for compiled-in inference plugins.
    #[cfg(feature = "inference-plugins")]
    pub version_managers: Arc<RwLock<HashMap<String, VersionManager>>>,
    /// Size calculator for release size estimates
    #[cfg(feature = "inference-plugins")]
    pub size_calculator: Arc<Mutex<SizeCalculator>>,
    /// Plugin configuration loader
    #[cfg(feature = "inference-plugins")]
    pub plugin_loader: Arc<PluginLoader>,
    /// Shared HTTP client for OpenAI-compatible gateway proxying.
    #[cfg(feature = "inference-plugins")]
    pub gateway_http_client: reqwest::Client,
    /// Public loopback base URL for the OpenAI-compatible serving gateway.
    #[cfg(feature = "inference-plugins")]
    pub gateway_base_url: RuntimeEndpointUrl,
    /// Runtime provider behavior registry for RPC boundary routing.
    #[cfg(feature = "inference-plugins")]
    pub provider_registry: ProviderRegistry,
    /// Shared llama.cpp router client for provider serving operations.
    #[cfg(feature = "inference-plugins")]
    pub llama_cpp_router_client: LlamaCppRouterClient,
    /// Shared Ollama client factory for provider serving and app operations.
    #[cfg(feature = "inference-plugins")]
    pub ollama_client_factory: OllamaClientFactory,
    /// Shared ONNX Runtime session manager for in-process embedding serving.
    #[cfg(feature = "inference-plugins")]
    pub onnx_session_manager: OnnxSessionManager<OnnxEmbeddingBackendKind>,
}

/// Owned handle for the running HTTP server task.
pub struct ServerHandle {
    addr: SocketAddr,
    task: Option<JoinHandle<anyhow::Result<()>>>,
    shutdown_signal: Option<oneshot::Sender<()>>,
}

impl ServerHandle {
    /// Address the server actually bound to.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Stop the server task and wait until it is no longer running.
    pub async fn shutdown(mut self) -> anyhow::Result<()> {
        if let Some(signal) = self.shutdown_signal.take() {
            let _ = signal.send(());
        }
        if let Some(task) = self.task.take() {
            task.await??;
        }
        Ok(())
    }
}

impl Drop for ServerHandle {
    fn drop(&mut self) {
        // Drop requests shutdown but cannot prove its asynchronous completion.
        // Call shutdown() for an observed drain; the supervisor owns its worker.
        if let Some(signal) = self.shutdown_signal.take() {
            let _ = signal.send(());
        }
    }
}

/// Start the JSON-RPC HTTP server.
///
/// Returns an owned handle that exposes the actual bound address and server task.
#[allow(clippy::too_many_arguments)]
pub async fn start_server(
    api: PumasApi,
    #[cfg(feature = "inference-plugins")] version_managers: HashMap<String, VersionManager>,
    #[cfg(feature = "inference-plugins")] size_calculator: SizeCalculator,
    #[cfg(feature = "inference-plugins")] plugin_loader: PluginLoader,
    host: LoopbackHost,
    port: u16,
) -> anyhow::Result<ServerHandle> {
    #[cfg(feature = "inference-plugins")]
    let gateway_http_client = build_gateway_http_client()?;
    #[cfg(feature = "inference-plugins")]
    let provider_http_client = build_provider_http_client()?;
    let addr = host.socket_addr(port);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let actual_addr = listener.local_addr()?;
    #[cfg(feature = "inference-plugins")]
    let gateway_base_url = RuntimeEndpointUrl::parse(format!("http://{actual_addr}/v1"))
        .map_err(|message| anyhow::anyhow!("invalid gateway base URL: {message}"))?;
    #[cfg(feature = "inference-plugins")]
    let ollama_client_factory = build_ollama_client_factory()?;
    #[cfg(feature = "inference-plugins")]
    let onnx_session_manager = OnnxSessionManager::new(
        OnnxEmbeddingBackendKind::real(),
        ONNX_MAX_CONCURRENT_OPERATIONS,
    )
    .map_err(|err| anyhow::anyhow!("failed to build ONNX session manager: {err}"))?;
    #[cfg(feature = "inference-plugins")]
    let provider_registry = ProviderRegistry::builtin();
    let (catalog_projection, catalog_worker) = CatalogProjection::start(MAX_IN_FLIGHT_RPC_REQUESTS);
    let state = Arc::new(AppState {
        catalog_projection,
        api,
        #[cfg(feature = "inference-plugins")]
        version_managers: Arc::new(RwLock::new(version_managers)),
        #[cfg(feature = "inference-plugins")]
        size_calculator: Arc::new(Mutex::new(size_calculator)),
        #[cfg(feature = "inference-plugins")]
        plugin_loader: Arc::new(plugin_loader),
        #[cfg(feature = "inference-plugins")]
        gateway_http_client,
        #[cfg(feature = "inference-plugins")]
        gateway_base_url,
        #[cfg(feature = "inference-plugins")]
        provider_registry,
        #[cfg(feature = "inference-plugins")]
        llama_cpp_router_client: LlamaCppRouterClient::new(provider_http_client),
        #[cfg(feature = "inference-plugins")]
        ollama_client_factory,
        #[cfg(feature = "inference-plugins")]
        onnx_session_manager,
    });

    // Configure CORS for local development and packaged renderer diagnostics.
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin, _parts| {
            is_allowed_cors_origin(origin)
        }))
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE]);

    // Build the router
    let app = Router::new()
        .route("/health", get(handle_health))
        .route(
            "/events/model-library-updates",
            get(handle_model_library_update_events),
        )
        .route(
            "/events/model-download-updates",
            get(handle_model_download_update_events),
        )
        .route(
            "/events/status-telemetry-updates",
            get(handle_status_telemetry_update_events),
        )
        .route("/rpc", post(handle_rpc));

    #[cfg(feature = "inference-plugins")]
    let app = app
        .route(
            "/events/runtime-profile-updates",
            get(handle_runtime_profile_update_events),
        )
        .route(
            "/events/serving-status-updates",
            get(handle_serving_status_update_events),
        )
        .route("/v1/models", get(handle_openai_models))
        .route("/v1/chat/completions", post(handle_openai_proxy))
        .route("/v1/completions", post(handle_openai_proxy))
        .route("/v1/embeddings", post(handle_openai_proxy));

    let app = app
        .layer(DefaultBodyLimit::max(MAX_REQUEST_BODY_BYTES))
        .layer(ConcurrencyLimitLayer::new(MAX_IN_FLIGHT_RPC_REQUESTS))
        .layer(cors)
        .with_state(state);

    info!(
        "Server listening on {} with max {} in-flight requests and {} byte request bodies",
        actual_addr, MAX_IN_FLIGHT_RPC_REQUESTS, MAX_REQUEST_BODY_BYTES
    );

    // Spawn the server in the background and retain ownership of the task.
    let (shutdown_signal, shutdown) = oneshot::channel();
    let task = tokio::spawn(async move {
        let serving = axum::serve(listener, app).into_future();
        let server_result = tokio::select! {
            result = serving => result.map_err(anyhow::Error::from),
            _ = shutdown => Ok(()),
        };
        let projection_result = catalog_worker.shutdown().await;
        server_result.and(projection_result)
    });

    Ok(ServerHandle {
        addr: actual_addr,
        task: Some(task),
        shutdown_signal: Some(shutdown_signal),
    })
}

#[cfg(feature = "inference-plugins")]
fn build_gateway_http_client() -> anyhow::Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .timeout(GATEWAY_PROXY_TIMEOUT)
        .build()?)
}

#[cfg(feature = "inference-plugins")]
fn build_provider_http_client() -> anyhow::Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .connect_timeout(PROVIDER_HTTP_CONNECT_TIMEOUT)
        .user_agent("pumas-library")
        .build()?)
}

#[cfg(feature = "inference-plugins")]
fn build_ollama_client_factory() -> anyhow::Result<OllamaClientFactory> {
    let http_clients = pumas_app_manager::OllamaHttpClients::new()
        .map_err(|err| anyhow::anyhow!("failed to build Ollama HTTP clients: {err}"))?;
    Ok(OllamaClientFactory::new(http_clients))
}

fn is_allowed_cors_origin(origin: &HeaderValue) -> bool {
    let Ok(origin) = origin.to_str() else {
        return false;
    };
    let Ok(url) = url::Url::parse(origin) else {
        return false;
    };
    if !matches!(url.scheme(), "http" | "https") {
        return false;
    }
    match url.host() {
        Some(url::Host::Domain("localhost")) => true,
        Some(url::Host::Ipv4(address)) => address.is_loopback(),
        Some(url::Host::Ipv6(address)) => address.is_loopback(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "inference-plugins")]
    use pumas_library::AppId;
    use std::io::ErrorKind;
    use tempfile::TempDir;

    #[test]
    fn loopback_host_rejects_every_remote_or_ambiguous_form() {
        assert!(LoopbackHost::parse("127.0.0.1").is_ok());
        assert!(LoopbackHost::parse("::1").is_ok());

        for host in ["0.0.0.0", "::", "192.168.1.10", "8.8.8.8", "localhost"] {
            let error = LoopbackHost::parse(host).err().expect("host must fail");
            assert!(error.to_string().contains("loopback"), "{host}: {error}");
        }
    }

    fn is_socket_bind_permission_error(err: &anyhow::Error) -> bool {
        err.chain().any(|cause| {
            cause
                .downcast_ref::<std::io::Error>()
                .map(|io_err| {
                    io_err.kind() == ErrorKind::PermissionDenied || io_err.raw_os_error() == Some(1)
                })
                .unwrap_or(false)
        })
    }

    #[tokio::test]
    async fn test_server_starts() {
        let temp_dir = TempDir::new().unwrap();
        let launcher_root = temp_dir.path().to_path_buf();
        let api = PumasApi::new(&launcher_root).await.unwrap();

        #[cfg(feature = "inference-plugins")]
        let mut version_managers = HashMap::new();
        #[cfg(feature = "inference-plugins")]
        if let Ok(vm) = VersionManager::new(&launcher_root, AppId::Ollama).await {
            version_managers.insert("ollama".to_string(), vm);
        }

        #[cfg(feature = "inference-plugins")]
        let cache_dir = launcher_root.join("launcher-data").join("cache");
        #[cfg(feature = "inference-plugins")]
        let size_calculator = SizeCalculator::new_with_cache(cache_dir).await;

        #[cfg(feature = "inference-plugins")]
        let plugins_dir = launcher_root.join("launcher-data").join("plugins");
        #[cfg(feature = "inference-plugins")]
        let plugin_loader = PluginLoader::new_async(plugins_dir).await.unwrap();

        let result = start_server(
            api,
            #[cfg(feature = "inference-plugins")]
            version_managers,
            #[cfg(feature = "inference-plugins")]
            size_calculator,
            #[cfg(feature = "inference-plugins")]
            plugin_loader,
            LoopbackHost::parse("127.0.0.1").unwrap(),
            0,
        )
        .await;
        let server = match result {
            Ok(server) => server,
            Err(err) if is_socket_bind_permission_error(&err) => {
                eprintln!("Skipping test_server_starts: socket bind not permitted ({err})");
                return;
            }
            Err(err) => panic!("test_server_starts failed: {err:#}"),
        };
        let addr = server.addr();
        assert!(addr.port() > 0);
        server.shutdown().await.unwrap();
    }

    #[test]
    fn cors_allows_loopback_origins() {
        for origin in [
            "http://localhost:5173",
            "http://127.0.0.1:5173",
            "http://[::1]:5173",
        ] {
            let header = HeaderValue::from_str(origin).unwrap();
            assert!(is_allowed_cors_origin(&header), "{origin}");
        }
    }

    #[test]
    fn cors_rejects_non_loopback_origins() {
        for origin in [
            "https://example.com",
            "http://192.168.1.10:5173",
            "file:///tmp/index.html",
        ] {
            let header = HeaderValue::from_str(origin).unwrap();
            assert!(!is_allowed_cors_origin(&header), "{origin}");
        }
    }

    #[cfg(feature = "inference-plugins")]
    #[test]
    fn gateway_http_client_builds_with_configured_policy() {
        build_gateway_http_client().unwrap();
    }
}
