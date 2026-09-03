//! JSON-RPC request handlers, split by domain.

mod conversion;
mod links;
mod models;
#[cfg(feature = "inference-plugins")]
mod ollama;
#[cfg(feature = "inference-plugins")]
mod openai_gateway;
#[cfg(feature = "inference-plugins")]
mod openai_gateway_onnx;
#[cfg(feature = "inference-plugins")]
mod plugins;
mod process;
#[cfg(feature = "inference-plugins")]
mod runtime_profiles;
#[cfg(feature = "inference-plugins")]
mod serving;
#[cfg(feature = "inference-plugins")]
mod serving_llama_cpp;
#[cfg(feature = "inference-plugins")]
mod serving_llama_cpp_router;
#[cfg(feature = "inference-plugins")]
mod serving_llama_cpp_shared;
#[cfg(feature = "inference-plugins")]
mod serving_ollama;
#[cfg(feature = "inference-plugins")]
mod serving_onnx;
mod shared;
mod status;
#[cfg(all(test, feature = "inference-plugins"))]
mod test_support;
#[cfg(feature = "inference-plugins")]
mod torch;
#[cfg(feature = "inference-plugins")]
mod versions;

use crate::server::AppState;
use crate::wrapper::wrap_response;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    Json,
};
use futures::{
    stream::{self, BoxStream},
    StreamExt,
};
use pumas_library::models::{
    ModelDownloadUpdateNotification, ModelLibraryUpdateNotification,
    StatusTelemetryUpdateNotification,
};
#[cfg(feature = "inference-plugins")]
use pumas_library::models::{RuntimeProfileUpdateFeed, ServingStatusUpdateFeed};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, warn};

const MODEL_LIBRARY_UPDATE_STREAM_LIMIT: usize = 250;

#[cfg(feature = "inference-plugins")]
pub use openai_gateway::{handle_openai_models, handle_openai_proxy};

pub(crate) use shared::{
    detect_sandbox_environment, extract_safetensors_header, get_bool_param, get_i64_param,
    get_str_param, parse_params, path_exists, require_str_param,
    validate_existing_local_directory_path, validate_existing_local_file_path,
    validate_existing_local_path, validate_external_url, validate_local_write_target_path,
    validate_non_empty,
};
#[cfg(feature = "inference-plugins")]
pub(crate) use shared::{get_version_manager, read_utf8_file, require_version_manager};

// ============================================================================
// JSON-RPC types
// ============================================================================

/// JSON-RPC 2.0 request structure.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
    pub id: Option<Value>,
}

/// JSON-RPC 2.0 response structure.
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: Option<Value>,
}

/// JSON-RPC 2.0 error structure.
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn error(id: Option<Value>, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
            id,
        }
    }
}

// ============================================================================
// HTTP endpoints
// ============================================================================

/// Health check endpoint.
pub async fn handle_health() -> impl IntoResponse {
    Json(json!({"status": "ok"}))
}

/// Server-sent model-library update notification stream.
pub async fn handle_model_library_update_events(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ModelLibraryUpdateStreamQuery>,
) -> Sse<BoxStream<'static, Result<Event, Infallible>>> {
    let stream: BoxStream<'static, Result<Event, Infallible>> =
        match build_model_library_update_stream_state(state, query.cursor).await {
            Ok(stream_state) => {
                stream::unfold(stream_state, next_model_library_update_event).boxed()
            }
            Err(error) => {
                warn!("model-library update stream startup failed: {}", error);
                stream::once(async move {
                    Ok(Event::default()
                        .event("model-library-error")
                        .data(json!({ "error": error.to_string() }).to_string()))
                })
                .boxed()
            }
        };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

#[derive(Debug, Default, Deserialize)]
pub struct ModelLibraryUpdateStreamQuery {
    pub cursor: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct ModelDownloadUpdateStreamQuery {
    pub cursor: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[cfg(feature = "inference-plugins")]
pub struct RuntimeProfileUpdateStreamQuery {
    pub cursor: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[cfg(feature = "inference-plugins")]
pub struct ServingStatusUpdateStreamQuery {
    pub cursor: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct StatusTelemetryUpdateStreamQuery {
    pub cursor: Option<String>,
}

async fn build_model_library_update_stream_state(
    state: Arc<AppState>,
    cursor: Option<String>,
) -> pumas_library::Result<ModelLibraryUpdateStreamState> {
    let requested_cursor = match cursor {
        Some(cursor) if !cursor.trim().is_empty() => cursor,
        _ => {
            state
                .api
                .list_model_library_updates_since(None, MODEL_LIBRARY_UPDATE_STREAM_LIMIT)
                .await?
                .cursor
        }
    };
    let subscriber = state
        .api
        .subscribe_model_library_update_stream_since(&requested_cursor)
        .await?;
    let handshake = subscriber.handshake().clone();
    let cursor = handshake.cursor_after_recovery.clone();
    let pending_notification = if handshake.recovered_events.is_empty()
        && !handshake.stale_cursor
        && !handshake.snapshot_required
    {
        None
    } else {
        Some(ModelLibraryUpdateNotification {
            cursor: handshake.cursor_after_recovery,
            events: handshake.recovered_events,
            stale_cursor: handshake.stale_cursor,
            snapshot_required: handshake.snapshot_required,
        })
    };

    Ok(ModelLibraryUpdateStreamState {
        subscriber,
        cursor,
        pending_notification,
    })
}

/// Server-sent model download update notification stream.
pub async fn handle_model_download_update_events(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ModelDownloadUpdateStreamQuery>,
) -> Sse<BoxStream<'static, Result<Event, Infallible>>> {
    let stream: BoxStream<'static, Result<Event, Infallible>> =
        match build_model_download_update_stream_state(state, query.cursor).await {
            Ok(stream_state) => {
                stream::unfold(stream_state, next_model_download_update_event).boxed()
            }
            Err(error) => {
                warn!("model download update stream startup failed: {}", error);
                stream::once(async move {
                    Ok(Event::default()
                        .event("model-download-error")
                        .data(json!({ "error": error.to_string() }).to_string()))
                })
                .boxed()
            }
        };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn build_model_download_update_stream_state(
    state: Arc<AppState>,
    cursor: Option<String>,
) -> pumas_library::Result<ModelDownloadUpdateStreamState> {
    let receiver = state.api.subscribe_hf_download_updates().ok_or_else(|| {
        pumas_library::PumasError::Config {
            message: "HuggingFace client not initialized".to_string(),
        }
    })?;
    let pending_notification = Some(
        state
            .api
            .hf_download_notification_since(cursor.as_deref())
            .await,
    );

    Ok(ModelDownloadUpdateStreamState {
        state,
        receiver,
        pending_notification,
    })
}

/// Server-sent runtime-profile update notification stream.
#[cfg(feature = "inference-plugins")]
pub async fn handle_runtime_profile_update_events(
    State(state): State<Arc<AppState>>,
    Query(query): Query<RuntimeProfileUpdateStreamQuery>,
) -> Sse<BoxStream<'static, Result<Event, Infallible>>> {
    let stream_state = build_runtime_profile_update_stream_state(state, query.cursor).await;
    let stream = stream::unfold(stream_state, next_runtime_profile_update_event).boxed();

    Sse::new(stream).keep_alive(KeepAlive::default())
}

#[cfg(feature = "inference-plugins")]
async fn build_runtime_profile_update_stream_state(
    state: Arc<AppState>,
    cursor: Option<String>,
) -> RuntimeProfileUpdateStreamState {
    let receiver = state.api.subscribe_runtime_profile_updates();
    let pending_feed = match state
        .api
        .list_runtime_profile_updates_since(cursor.as_deref())
        .await
    {
        Ok(response)
            if !response.feed.events.is_empty()
                || response.feed.stale_cursor
                || response.feed.snapshot_required =>
        {
            Some(response.feed)
        }
        Ok(_) => None,
        Err(error) => {
            warn!(
                "runtime-profile update stream startup recovery failed: {}",
                error
            );
            Some(RuntimeProfileUpdateFeed::snapshot_required(
                "runtime-profiles:0".to_string(),
            ))
        }
    };

    RuntimeProfileUpdateStreamState {
        receiver,
        pending_feed,
    }
}

/// Server-sent serving-status update notification stream.
#[cfg(feature = "inference-plugins")]
pub async fn handle_serving_status_update_events(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ServingStatusUpdateStreamQuery>,
) -> Sse<BoxStream<'static, Result<Event, Infallible>>> {
    let stream_state = build_serving_status_update_stream_state(state, query.cursor).await;
    let stream = stream::unfold(stream_state, next_serving_status_update_event).boxed();

    Sse::new(stream).keep_alive(KeepAlive::default())
}

#[cfg(feature = "inference-plugins")]
async fn build_serving_status_update_stream_state(
    state: Arc<AppState>,
    cursor: Option<String>,
) -> ServingStatusUpdateStreamState {
    let receiver = state.api.subscribe_serving_status_updates();
    let pending_feed = match state
        .api
        .list_serving_status_updates_since(cursor.as_deref())
        .await
    {
        Ok(response)
            if !response.feed.events.is_empty()
                || response.feed.stale_cursor
                || response.feed.snapshot_required =>
        {
            Some(response.feed)
        }
        Ok(_) => None,
        Err(error) => {
            warn!(
                "serving-status update stream startup recovery failed: {}",
                error
            );
            Some(ServingStatusUpdateFeed::snapshot_required(
                "serving:0".to_string(),
            ))
        }
    };

    ServingStatusUpdateStreamState {
        receiver,
        pending_feed,
    }
}

/// Server-sent status/resource telemetry update stream.
pub async fn handle_status_telemetry_update_events(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StatusTelemetryUpdateStreamQuery>,
) -> Sse<BoxStream<'static, Result<Event, Infallible>>> {
    let stream: BoxStream<'static, Result<Event, Infallible>> =
        match build_status_telemetry_update_stream_state(state, query.cursor).await {
            Ok(stream_state) => {
                stream::unfold(stream_state, next_status_telemetry_update_event).boxed()
            }
            Err(error) => {
                warn!("status telemetry update stream startup failed: {}", error);
                stream::once(async move {
                    Ok(Event::default()
                        .event("status-telemetry-error")
                        .data(json!({ "error": error.to_string() }).to_string()))
                })
                .boxed()
            }
        };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Main JSON-RPC handler.
pub async fn handle_rpc(
    State(state): State<Arc<AppState>>,
    Json(request): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    let method = &request.method;
    let params = request.params.unwrap_or(Value::Object(Default::default()));
    let id = request.id.clone();

    debug!("RPC call: {}({:?})", method, params);

    // Handle built-in methods
    if method == "health_check" {
        return (
            StatusCode::OK,
            Json(JsonRpcResponse::success(id, json!({"status": "ok"}))),
        );
    }

    if method == "shutdown" {
        #[cfg(not(feature = "inference-plugins"))]
        return (
            StatusCode::OK,
            Json(JsonRpcResponse::success(
                id,
                json!({ "status": "shutting_down" }),
            )),
        );

        #[cfg(feature = "inference-plugins")]
        let shutdown_summary = match state.api.stop_all_managed_runtime_profiles().await {
            Ok(summary) => summary,
            Err(error) => {
                warn!(
                    "managed runtime shutdown failed before backend exit: {}",
                    error
                );
                return (
                    StatusCode::OK,
                    Json(JsonRpcResponse::success(
                        id,
                        json!({
                            "status": "shutting_down",
                            "managed_profiles_processed": 0,
                            "managed_processes_stopped": 0,
                            "errors": [error.to_string()],
                        }),
                    )),
                );
            }
        };
        #[cfg(feature = "inference-plugins")]
        return (
            StatusCode::OK,
            Json(JsonRpcResponse::success(
                id,
                json!({
                    "status": "shutting_down",
                    "managed_profiles_processed": shutdown_summary.profiles_processed,
                    "managed_processes_stopped": shutdown_summary.processes_stopped,
                    "errors": shutdown_summary.errors,
                }),
            )),
        );
    }

    // Dispatch to API methods
    let result = dispatch_method(&state, method, &params).await;

    match result {
        Ok(value) => {
            let wrapped = wrap_response(method, value);
            (StatusCode::OK, Json(JsonRpcResponse::success(id, wrapped)))
        }
        Err(e) => {
            error!("RPC error for {}: {}", method, e);
            let code = e.to_rpc_error_code();
            (
                StatusCode::OK,
                Json(JsonRpcResponse::error(id, code, e.to_string())),
            )
        }
    }
}

struct ModelLibraryUpdateStreamState {
    subscriber: pumas_library::model_library::ModelLibraryUpdateSubscriber,
    cursor: String,
    pending_notification: Option<ModelLibraryUpdateNotification>,
}

struct ModelDownloadUpdateStreamState {
    state: Arc<AppState>,
    receiver: broadcast::Receiver<ModelDownloadUpdateNotification>,
    pending_notification: Option<ModelDownloadUpdateNotification>,
}

async fn next_model_library_update_event(
    mut state: ModelLibraryUpdateStreamState,
) -> Option<(Result<Event, Infallible>, ModelLibraryUpdateStreamState)> {
    if let Some(notification) = state.pending_notification.take() {
        let event = model_library_update_sse_event(&notification);
        return Some((Ok(event), state));
    }

    match state.subscriber.next_event().await {
        Ok(update) => {
            state.cursor = update.cursor.clone();
            let notification = ModelLibraryUpdateNotification {
                cursor: update.cursor.clone(),
                events: vec![update],
                stale_cursor: false,
                snapshot_required: false,
            };
            let event = model_library_update_sse_event(&notification);
            Some((Ok(event), state))
        }
        Err(error) => {
            warn!(
                cursor = %state.cursor,
                "model-library update stream ended: {}",
                error
            );
            None
        }
    }
}

fn model_library_update_sse_event(notification: &ModelLibraryUpdateNotification) -> Event {
    match serde_json::to_string(notification) {
        Ok(payload) => Event::default().event("model-library-update").data(payload),
        Err(error) => Event::default()
            .event("model-library-error")
            .data(json!({ "error": error.to_string() }).to_string()),
    }
}

async fn next_model_download_update_event(
    mut state: ModelDownloadUpdateStreamState,
) -> Option<(Result<Event, Infallible>, ModelDownloadUpdateStreamState)> {
    if let Some(notification) = state.pending_notification.take() {
        let event = model_download_update_sse_event(&notification);
        return Some((Ok(event), state));
    }

    match state.receiver.recv().await {
        Ok(notification) => {
            let event = model_download_update_sse_event(&notification);
            Some((Ok(event), state))
        }
        Err(broadcast::error::RecvError::Lagged(_)) => {
            let notification = state.state.api.hf_download_notification_since(None).await;
            let event = model_download_update_sse_event(&notification);
            Some((Ok(event), state))
        }
        Err(broadcast::error::RecvError::Closed) => None,
    }
}

fn model_download_update_sse_event(notification: &ModelDownloadUpdateNotification) -> Event {
    match serde_json::to_string(notification) {
        Ok(payload) => Event::default()
            .event("model-download-update")
            .data(payload),
        Err(error) => Event::default()
            .event("model-download-error")
            .data(json!({ "error": error.to_string() }).to_string()),
    }
}

#[cfg(feature = "inference-plugins")]
struct RuntimeProfileUpdateStreamState {
    receiver: broadcast::Receiver<RuntimeProfileUpdateFeed>,
    pending_feed: Option<RuntimeProfileUpdateFeed>,
}

#[cfg(feature = "inference-plugins")]
struct ServingStatusUpdateStreamState {
    receiver: broadcast::Receiver<ServingStatusUpdateFeed>,
    pending_feed: Option<ServingStatusUpdateFeed>,
}

struct StatusTelemetryUpdateStreamState {
    state: Arc<AppState>,
    receiver: broadcast::Receiver<StatusTelemetryUpdateNotification>,
    pending_notification: Option<StatusTelemetryUpdateNotification>,
}

async fn build_status_telemetry_update_stream_state(
    state: Arc<AppState>,
    cursor: Option<String>,
) -> pumas_library::Result<StatusTelemetryUpdateStreamState> {
    let receiver = state.api.subscribe_status_telemetry_updates();
    let snapshot = state.api.get_status_telemetry_snapshot().await?;
    let pending_notification = state
        .api
        .status_telemetry_notification_since(cursor.as_deref(), snapshot);

    Ok(StatusTelemetryUpdateStreamState {
        state,
        receiver,
        pending_notification,
    })
}

async fn next_status_telemetry_update_event(
    mut state: StatusTelemetryUpdateStreamState,
) -> Option<(Result<Event, Infallible>, StatusTelemetryUpdateStreamState)> {
    if let Some(notification) = state.pending_notification.take() {
        let event = status_telemetry_update_sse_event(&notification);
        return Some((Ok(event), state));
    }

    match state.receiver.recv().await {
        Ok(notification) => {
            let event = status_telemetry_update_sse_event(&notification);
            Some((Ok(event), state))
        }
        Err(broadcast::error::RecvError::Lagged(_)) => {
            match state.state.api.get_status_telemetry_snapshot().await {
                Ok(snapshot) => {
                    let notification = StatusTelemetryUpdateNotification {
                        cursor: snapshot.cursor.clone(),
                        snapshot,
                        stale_cursor: true,
                        snapshot_required: true,
                    };
                    let event = status_telemetry_update_sse_event(&notification);
                    Some((Ok(event), state))
                }
                Err(error) => {
                    warn!("status telemetry refresh after lag failed: {}", error);
                    None
                }
            }
        }
        Err(broadcast::error::RecvError::Closed) => None,
    }
}

fn status_telemetry_update_sse_event(notification: &StatusTelemetryUpdateNotification) -> Event {
    match serde_json::to_string(notification) {
        Ok(payload) => Event::default()
            .event("status-telemetry-update")
            .data(payload),
        Err(error) => Event::default()
            .event("status-telemetry-error")
            .data(json!({ "error": error.to_string() }).to_string()),
    }
}

#[cfg(feature = "inference-plugins")]
async fn next_runtime_profile_update_event(
    mut state: RuntimeProfileUpdateStreamState,
) -> Option<(Result<Event, Infallible>, RuntimeProfileUpdateStreamState)> {
    if let Some(feed) = state.pending_feed.take() {
        let event = runtime_profile_update_sse_event(&feed);
        return Some((Ok(event), state));
    }

    match state.receiver.recv().await {
        Ok(feed) => {
            let event = runtime_profile_update_sse_event(&feed);
            Some((Ok(event), state))
        }
        Err(broadcast::error::RecvError::Lagged(_)) => {
            let feed =
                RuntimeProfileUpdateFeed::snapshot_required("runtime-profiles:0".to_string());
            let event = runtime_profile_update_sse_event(&feed);
            Some((Ok(event), state))
        }
        Err(broadcast::error::RecvError::Closed) => None,
    }
}

#[cfg(feature = "inference-plugins")]
fn runtime_profile_update_sse_event(feed: &RuntimeProfileUpdateFeed) -> Event {
    match serde_json::to_string(feed) {
        Ok(payload) => Event::default()
            .event("runtime-profile-update")
            .data(payload),
        Err(error) => Event::default()
            .event("runtime-profile-error")
            .data(json!({ "error": error.to_string() }).to_string()),
    }
}

#[cfg(feature = "inference-plugins")]
async fn next_serving_status_update_event(
    mut state: ServingStatusUpdateStreamState,
) -> Option<(Result<Event, Infallible>, ServingStatusUpdateStreamState)> {
    if let Some(feed) = state.pending_feed.take() {
        let event = serving_status_update_sse_event(&feed);
        return Some((Ok(event), state));
    }

    match state.receiver.recv().await {
        Ok(feed) => {
            let event = serving_status_update_sse_event(&feed);
            Some((Ok(event), state))
        }
        Err(broadcast::error::RecvError::Lagged(_)) => {
            let feed = ServingStatusUpdateFeed::snapshot_required("serving:0".to_string());
            let event = serving_status_update_sse_event(&feed);
            Some((Ok(event), state))
        }
        Err(broadcast::error::RecvError::Closed) => None,
    }
}

#[cfg(feature = "inference-plugins")]
fn serving_status_update_sse_event(feed: &ServingStatusUpdateFeed) -> Event {
    match serde_json::to_string(feed) {
        Ok(payload) => Event::default()
            .event("serving-status-update")
            .data(payload),
        Err(error) => Event::default()
            .event("serving-status-error")
            .data(json!({ "error": error.to_string() }).to_string()),
    }
}

// ============================================================================
// Method dispatcher
// ============================================================================

/// Dispatch a method call to the appropriate domain handler.
async fn dispatch_method(
    state: &AppState,
    method: &str,
    params: &Value,
) -> pumas_library::Result<Value> {
    match method {
        // Status & System
        "get_status" => status::get_status(state, params).await,
        "get_disk_space" => status::get_disk_space(state, params).await,
        "get_system_resources" => status::get_system_resources(state, params).await,
        "get_status_telemetry_snapshot" => {
            status::get_status_telemetry_snapshot(state, params).await
        }
        "get_launcher_version" => status::get_launcher_version(state, params).await,
        "check_launcher_updates" => status::check_launcher_updates(state, params).await,
        "apply_launcher_update" => status::apply_launcher_update(state, params).await,
        "restart_launcher" => status::restart_launcher(state, params).await,
        "get_sandbox_info" => status::get_sandbox_info(state, params).await,
        "check_git" => status::check_git(state, params).await,
        "get_network_status" => status::get_network_status(state, params).await,
        "get_library_status" => status::get_library_status(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "get_app_status" => status::get_app_status(state, params).await,

        // Local Runtime Profiles
        #[cfg(feature = "inference-plugins")]
        "get_runtime_profiles_snapshot" => {
            runtime_profiles::get_runtime_profiles_snapshot(state, params).await
        }
        #[cfg(feature = "inference-plugins")]
        "list_runtime_profile_updates_since" => {
            runtime_profiles::list_runtime_profile_updates_since(state, params).await
        }
        #[cfg(feature = "inference-plugins")]
        "upsert_runtime_profile" => runtime_profiles::upsert_runtime_profile(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "delete_runtime_profile" => runtime_profiles::delete_runtime_profile(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "set_model_runtime_route" => runtime_profiles::set_model_runtime_route(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "clear_model_runtime_route" => {
            runtime_profiles::clear_model_runtime_route(state, params).await
        }
        #[cfg(feature = "inference-plugins")]
        "launch_runtime_profile" => runtime_profiles::launch_runtime_profile(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "stop_runtime_profile" => runtime_profiles::stop_runtime_profile(state, params).await,

        // User-Directed Serving
        #[cfg(feature = "inference-plugins")]
        "get_serving_status" => serving::get_serving_status(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "list_serving_status_updates_since" => {
            serving::list_serving_status_updates_since(state, params).await
        }
        #[cfg(feature = "inference-plugins")]
        "validate_model_serving_config" => {
            serving::validate_model_serving_config(state, params).await
        }
        #[cfg(feature = "inference-plugins")]
        "serve_model" => serving::serve_model(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "unserve_model" => serving::unserve_model(state, params).await,

        // Version Management
        #[cfg(feature = "inference-plugins")]
        "get_available_versions" => versions::get_available_versions(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "get_installed_versions" => versions::get_installed_versions(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "get_active_version" => versions::get_active_version(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "get_default_version" => versions::get_default_version(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "set_default_version" => versions::set_default_version(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "switch_version" => versions::switch_version(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "install_version" => versions::install_version(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "remove_version" => versions::remove_version(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "cancel_installation" => versions::cancel_installation(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "get_installation_progress" => versions::get_installation_progress(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "validate_installations" => versions::validate_installations(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "get_version_status" => versions::get_version_status(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "get_version_info" => versions::get_version_info(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "get_release_size_info" => versions::get_release_size_info(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "get_release_size_breakdown" => versions::get_release_size_breakdown(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "calculate_release_size" => versions::calculate_release_size(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "calculate_all_release_sizes" => versions::calculate_all_release_sizes(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "has_background_fetch_completed" => {
            versions::has_background_fetch_completed(state, params).await
        }
        #[cfg(feature = "inference-plugins")]
        "reset_background_fetch_flag" => versions::reset_background_fetch_flag(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "get_github_cache_status" => versions::get_github_cache_status(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "check_version_dependencies" => versions::check_version_dependencies(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "install_version_dependencies" => {
            versions::install_version_dependencies(state, params).await
        }
        #[cfg(feature = "inference-plugins")]
        "get_release_dependencies" => versions::get_release_dependencies(state, params).await,

        // Model Library
        "get_models" => models::get_models(state, params).await,
        "refresh_model_index" => models::refresh_model_index(state, params).await,
        "import_model" => models::import_model(state, params).await,
        "download_model_from_hf" => models::download_model_from_hf(state, params).await,
        "start_model_download_from_hf" => models::start_model_download_from_hf(state, params).await,
        "get_model_download_status" => models::get_model_download_status(state, params).await,
        "cancel_model_download" => models::cancel_model_download(state, params).await,
        "pause_model_download" => models::pause_model_download(state, params).await,
        "resume_model_download" => models::resume_model_download(state, params).await,
        "list_model_downloads" => models::list_model_downloads(state, params).await,
        "list_interrupted_downloads" => models::list_interrupted_downloads(state, params).await,
        "recover_download" => models::recover_download(state, params).await,
        "resume_partial_download" => models::resume_partial_download(state, params).await,
        "search_hf_models" => models::search_hf_models(state, params).await,
        "get_hf_download_details" => models::get_hf_download_details(state, params).await,
        "get_related_models" => models::get_related_models(state, params).await,
        "search_models_fts" => models::search_models_fts(state, params).await,
        "import_batch" => models::import_batch(state, params).await,
        "import_external_diffusers_directory" => {
            models::import_external_diffusers_directory(state, params).await
        }
        "classify_model_import_paths" => models::classify_model_import_paths(state, params).await,
        "lookup_hf_metadata_for_file" => models::lookup_hf_metadata_for_file(state, params).await,
        "lookup_hf_metadata_for_bundle_directory" => {
            models::lookup_hf_metadata_for_bundle_directory(state, params).await
        }
        "detect_sharded_sets" => models::detect_sharded_sets(state, params).await,
        "validate_file_type" => models::validate_file_type(state, params).await,
        "get_embedded_metadata" => models::get_embedded_metadata(state, params).await,
        "get_library_model_metadata" => models::get_library_model_metadata(state, params).await,
        "resolve_model_execution_descriptor" => {
            models::resolve_model_execution_descriptor(state, params).await
        }
        "resolve_model_artifact_load_target" => {
            models::resolve_model_artifact_load_target(state, params).await
        }
        "resolve_model_package_facts" => models::resolve_model_package_facts(state, params).await,
        "list_model_library_updates_since" => {
            models::list_model_library_updates_since(state, params).await
        }
        "resolve_model_package_facts_summary" => {
            models::resolve_model_package_facts_summary(state, params).await
        }
        "model_package_facts_summary_snapshot" => {
            models::model_package_facts_summary_snapshot(state, params).await
        }
        "refetch_model_metadata_from_hf" => {
            models::refetch_model_metadata_from_hf(state, params).await
        }
        "adopt_orphan_models" => models::adopt_orphan_models(state, params).await,
        "import_model_in_place" => models::import_model_in_place(state, params).await,
        "scan_shared_storage" => models::scan_shared_storage(state, params).await,

        // Inference Settings
        "get_inference_settings" => models::get_inference_settings(state, params).await,
        "update_inference_settings" => models::update_inference_settings(state, params).await,
        "update_model_notes" => models::update_model_notes(state, params).await,
        "resolve_model_dependency_requirements" => {
            models::resolve_model_dependency_requirements(state, params).await
        }
        "audit_dependency_pin_compliance" => {
            models::audit_dependency_pin_compliance(state, params).await
        }
        "list_models_needing_review" => models::list_models_needing_review(state, params).await,
        "submit_model_review" => models::submit_model_review(state, params).await,
        "reset_model_review" => models::reset_model_review(state, params).await,
        "generate_model_migration_dry_run_report" => {
            models::generate_model_migration_dry_run_report(state, params).await
        }
        "execute_model_migration" => models::execute_model_migration(state, params).await,
        "list_model_migration_reports" => models::list_model_migration_reports(state, params).await,
        "delete_model_migration_report" => {
            models::delete_model_migration_report(state, params).await
        }
        "prune_model_migration_reports" => {
            models::prune_model_migration_reports(state, params).await
        }

        // HuggingFace Authentication
        "set_hf_token" => models::set_hf_token(state, params).await,
        "clear_hf_token" => models::clear_hf_token(state, params).await,
        "get_hf_auth_status" => models::get_hf_auth_status(state, params).await,

        // Process Management
        #[cfg(feature = "inference-plugins")]
        "launch_ollama" => process::launch_ollama(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "stop_ollama" => process::stop_ollama(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "is_ollama_running" => process::is_ollama_running(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "launch_torch" => process::launch_torch(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "stop_torch" => process::stop_torch(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "is_torch_running" => process::is_torch_running(state, params).await,
        "open_path" => process::open_path(state, params).await,
        "open_url" => process::open_url(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "open_active_install" => process::open_active_install(state, params).await,

        // Ollama Model Management
        #[cfg(feature = "inference-plugins")]
        "ollama_list_models" => ollama::ollama_list_models(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "ollama_list_models_for_profile" => {
            ollama::ollama_list_models_for_profile(state, params).await
        }
        #[cfg(feature = "inference-plugins")]
        "ollama_create_model" => ollama::ollama_create_model(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "ollama_create_model_for_profile" => {
            ollama::ollama_create_model_for_profile(state, params).await
        }
        #[cfg(feature = "inference-plugins")]
        "ollama_delete_model" => ollama::ollama_delete_model(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "ollama_delete_model_for_profile" => {
            ollama::ollama_delete_model_for_profile(state, params).await
        }
        #[cfg(feature = "inference-plugins")]
        "ollama_load_model" => ollama::ollama_load_model(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "ollama_load_model_for_profile" => {
            ollama::ollama_load_model_for_profile(state, params).await
        }
        #[cfg(feature = "inference-plugins")]
        "ollama_unload_model" => ollama::ollama_unload_model(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "ollama_unload_model_for_profile" => {
            ollama::ollama_unload_model_for_profile(state, params).await
        }
        #[cfg(feature = "inference-plugins")]
        "ollama_list_running" => ollama::ollama_list_running(state, params).await,

        // Torch Inference Server
        #[cfg(feature = "inference-plugins")]
        "torch_list_slots" => torch::torch_list_slots(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "torch_load_model" => torch::torch_load_model(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "torch_unload_model" => torch::torch_unload_model(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "torch_get_status" => torch::torch_get_status(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "torch_list_devices" => torch::torch_list_devices(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "torch_configure" => torch::torch_configure(state, params).await,

        // Link Management
        "get_link_health" => links::get_link_health(state, params).await,
        "clean_broken_links" => links::clean_broken_links(state, params).await,
        "remove_orphaned_links" => links::remove_orphaned_links(state, params).await,
        "get_links_for_model" => links::get_links_for_model(state, params).await,
        "delete_model_with_cascade" => links::delete_model_with_cascade(state, params).await,
        "get_file_link_count" => links::get_file_link_count(state, params).await,
        "check_files_writable" => links::check_files_writable(state, params).await,
        "set_model_link_exclusion" => links::set_model_link_exclusion(state, params).await,
        "get_link_exclusions" => links::get_link_exclusions(state, params).await,

        // Conversion
        "start_model_conversion" => conversion::start_model_conversion(state, params).await,
        "get_conversion_progress" => conversion::get_conversion_progress(state, params).await,
        "cancel_model_conversion" => conversion::cancel_model_conversion(state, params).await,
        "list_model_conversions" => conversion::list_model_conversions(state, params).await,
        "check_conversion_environment" => {
            conversion::check_conversion_environment(state, params).await
        }
        "setup_conversion_environment" => {
            conversion::setup_conversion_environment(state, params).await
        }
        "get_supported_quant_types" => conversion::get_supported_quant_types(state, params).await,
        "get_backend_status" => conversion::get_backend_status(state, params).await,
        "setup_quantization_backend" => conversion::setup_quantization_backend(state, params).await,

        // Plugins
        #[cfg(feature = "inference-plugins")]
        "get_plugins" => plugins::get_plugins(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "get_plugin" => plugins::get_plugin(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "call_plugin_endpoint" => plugins::call_plugin_endpoint(state, params).await,
        #[cfg(feature = "inference-plugins")]
        "check_plugin_health" => plugins::check_plugin_health(state, params).await,

        // Unknown method
        _ => {
            warn!("Method not found: {}", method);
            Err(pumas_library::PumasError::Other(format!(
                "Method not found: {}",
                method
            )))
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_rpc_response_success() {
        let response = JsonRpcResponse::success(Some(json!(1)), json!({"data": "test"}));
        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }

    #[test]
    fn test_json_rpc_response_error() {
        let response = JsonRpcResponse::error(Some(json!(1)), -32600, "Test error".into());
        assert!(response.error.is_some());
        assert!(response.result.is_none());
        assert_eq!(response.error.unwrap().code, -32600);
    }

    #[tokio::test]
    async fn test_detect_sandbox() {
        let (is_sandboxed, sandbox_type, _) = detect_sandbox_environment().await;
        // In normal development, we're not sandboxed
        // This test verifies the function runs without error
        assert!(!is_sandboxed || ["flatpak", "snap", "docker", "appimage"].contains(&sandbox_type));
    }
}
