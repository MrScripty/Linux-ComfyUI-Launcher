//! Status & system check handlers.

use super::detect_sandbox_environment;
#[cfg(feature = "inference-plugins")]
use crate::contract::AppStatusOutcome;
use crate::contract::{LauncherVersionOutcome, OperationStatusOutcome, SandboxOutcome};
use crate::server::AppState;
use pumas_library::models::{
    DiskSpaceResponse, LibraryStatusResponse, NetworkStatusResponse, StatusResponse,
    StatusTelemetrySnapshot, SystemResourcesResponse,
};
use pumas_library::{SystemCheckResult, UpdateApplyResult, UpdateCheckResult};

pub async fn get_status(state: &AppState) -> pumas_library::Result<StatusResponse> {
    enriched_status_response(state).await
}

async fn enrich_status_telemetry_snapshot(
    state: &AppState,
    mut snapshot: StatusTelemetrySnapshot,
) -> pumas_library::Result<StatusTelemetrySnapshot> {
    snapshot.status = enriched_status_response(state).await?;
    Ok(snapshot)
}

async fn enriched_status_response(state: &AppState) -> pumas_library::Result<StatusResponse> {
    state.api.get_status().await
}

pub async fn get_disk_space(state: &AppState) -> pumas_library::Result<DiskSpaceResponse> {
    state.api.get_disk_space().await
}

pub async fn get_system_resources(
    state: &AppState,
) -> pumas_library::Result<SystemResourcesResponse> {
    state.api.get_system_resources().await
}

pub async fn get_status_telemetry_snapshot(
    state: &AppState,
) -> pumas_library::Result<StatusTelemetrySnapshot> {
    let snapshot = state.api.refresh_status_telemetry_snapshot().await?;
    enrich_status_telemetry_snapshot(state, snapshot).await
}

pub async fn get_launcher_version(
    state: &AppState,
) -> pumas_library::Result<LauncherVersionOutcome> {
    LauncherVersionOutcome::decode(state.api.get_launcher_version().await)
}

pub async fn check_launcher_updates(
    state: &AppState,
    force_refresh: bool,
) -> pumas_library::Result<UpdateCheckResult> {
    Ok(state.api.check_launcher_updates(force_refresh).await)
}

pub async fn apply_launcher_update(state: &AppState) -> pumas_library::Result<UpdateApplyResult> {
    Ok(state.api.apply_launcher_update().await)
}

pub async fn restart_launcher(state: &AppState) -> OperationStatusOutcome {
    match state.api.restart_launcher().await {
        Ok(true) => OperationStatusOutcome::success(),
        Ok(false) | Err(_) => OperationStatusOutcome::failed(),
    }
}

pub async fn get_sandbox_info() -> SandboxOutcome {
    let (is_sandboxed, sandbox_type, limitations) = detect_sandbox_environment().await;
    SandboxOutcome::new(is_sandboxed, sandbox_type, limitations)
}

pub async fn check_git(state: &AppState) -> SystemCheckResult {
    state.api.check_git().await
}

pub async fn get_network_status(state: &AppState) -> NetworkStatusResponse {
    state.api.get_network_status_response().await
}

pub async fn get_library_status(state: &AppState) -> pumas_library::Result<LibraryStatusResponse> {
    state.api.get_library_status().await
}

#[cfg(feature = "inference-plugins")]
pub async fn get_app_status(state: &AppState, app_id: &str) -> AppStatusOutcome {
    let running = match app_id {
        "ollama" => state.api.is_ollama_running().await,
        "torch" => state.api.is_torch_running().await,
        _ => false,
    };
    AppStatusOutcome::new(running)
}
