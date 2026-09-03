//! Status & system check handlers.

#[cfg(feature = "inference-plugins")]
use super::require_str_param;
use super::{detect_sandbox_environment, get_bool_param};
use crate::server::AppState;
use pumas_library::models::{StatusResponse, StatusTelemetrySnapshot};
use serde_json::{json, Value};

pub async fn get_status(state: &AppState, _params: &Value) -> pumas_library::Result<Value> {
    let response = enriched_status_response(state).await?;
    Ok(serde_json::to_value(response)?)
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

pub async fn get_disk_space(state: &AppState, _params: &Value) -> pumas_library::Result<Value> {
    let response = state.api.get_disk_space().await?;
    Ok(serde_json::to_value(response)?)
}

pub async fn get_system_resources(
    state: &AppState,
    _params: &Value,
) -> pumas_library::Result<Value> {
    let response = state.api.get_system_resources().await?;
    Ok(serde_json::to_value(response)?)
}

pub async fn get_status_telemetry_snapshot(
    state: &AppState,
    _params: &Value,
) -> pumas_library::Result<Value> {
    let snapshot = state.api.refresh_status_telemetry_snapshot().await?;
    let snapshot = enrich_status_telemetry_snapshot(state, snapshot).await?;
    Ok(serde_json::to_value(snapshot)?)
}

pub async fn get_launcher_version(
    state: &AppState,
    _params: &Value,
) -> pumas_library::Result<Value> {
    let version_info = state.api.get_launcher_version().await;
    Ok(version_info)
}

pub async fn check_launcher_updates(
    state: &AppState,
    params: &Value,
) -> pumas_library::Result<Value> {
    let force_refresh = get_bool_param(params, "force_refresh", "forceRefresh").unwrap_or(false);
    let result = state.api.check_launcher_updates(force_refresh).await;
    Ok(serde_json::to_value(result)?)
}

pub async fn apply_launcher_update(
    state: &AppState,
    _params: &Value,
) -> pumas_library::Result<Value> {
    let result = state.api.apply_launcher_update().await;
    Ok(serde_json::to_value(result)?)
}

pub async fn restart_launcher(state: &AppState, _params: &Value) -> pumas_library::Result<Value> {
    match state.api.restart_launcher().await {
        Ok(success) => Ok(json!({
            "success": success
        })),
        Err(e) => Ok(json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

pub async fn get_sandbox_info(_state: &AppState, _params: &Value) -> pumas_library::Result<Value> {
    let (is_sandboxed, sandbox_type, limitations) = detect_sandbox_environment().await;
    Ok(json!({
        "success": true,
        "is_sandboxed": is_sandboxed,
        "sandbox_type": sandbox_type,
        "limitations": limitations
    }))
}

pub async fn check_git(state: &AppState, _params: &Value) -> pumas_library::Result<Value> {
    let result = state.api.check_git().await;
    Ok(serde_json::to_value(result)?)
}

pub async fn get_network_status(state: &AppState, _params: &Value) -> pumas_library::Result<Value> {
    let status = state.api.get_network_status_response().await;
    Ok(serde_json::to_value(status)?)
}

pub async fn get_library_status(state: &AppState, _params: &Value) -> pumas_library::Result<Value> {
    let status = state.api.get_library_status().await?;
    Ok(serde_json::to_value(status)?)
}

#[cfg(feature = "inference-plugins")]
pub async fn get_app_status(state: &AppState, params: &Value) -> pumas_library::Result<Value> {
    let app_id = require_str_param(params, "app_id", "appId")?;
    let running = match app_id.as_str() {
        "ollama" => state.api.is_ollama_running().await,
        "torch" => state.api.is_torch_running().await,
        _ => false,
    };
    Ok(json!({
        "success": true,
        "running": running
    }))
}
