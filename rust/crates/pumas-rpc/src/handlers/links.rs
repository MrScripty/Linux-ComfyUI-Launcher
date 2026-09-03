//! Link management handlers.

use super::{
    get_str_param, path_exists, require_str_param, validate_existing_local_path,
    validate_local_write_target_path,
};
use crate::server::AppState;
use serde_json::{json, Value};
use std::path::PathBuf;

pub async fn get_link_health(state: &AppState, params: &Value) -> pumas_library::Result<Value> {
    let version_tag = get_str_param(params, "version_tag", "versionTag");
    let response = state.api.get_link_health(version_tag).await?;
    Ok(serde_json::to_value(response)?)
}

pub async fn clean_broken_links(state: &AppState, _params: &Value) -> pumas_library::Result<Value> {
    let response = state.api.clean_broken_links().await?;
    Ok(serde_json::to_value(response)?)
}

pub async fn remove_orphaned_links(
    state: &AppState,
    params: &Value,
) -> pumas_library::Result<Value> {
    let _version_tag = require_str_param(params, "version_tag", "versionTag")?;
    // Orphaned links are handled as part of clean_broken_links
    let response = state.api.clean_broken_links().await?;
    Ok(json!({
        "success": response.success,
        "removed": response.cleaned
    }))
}

pub async fn get_links_for_model(state: &AppState, params: &Value) -> pumas_library::Result<Value> {
    let model_id = require_str_param(params, "model_id", "modelId")?;
    let response = state.api.get_links_for_model(&model_id).await?;
    Ok(serde_json::to_value(response)?)
}

pub async fn delete_model_with_cascade(
    state: &AppState,
    params: &Value,
) -> pumas_library::Result<Value> {
    let model_id = require_str_param(params, "model_id", "modelId")?;
    let response = state.api.delete_model_with_cascade(&model_id).await?;
    Ok(serde_json::to_value(response)?)
}

pub async fn get_file_link_count(
    _state: &AppState,
    params: &Value,
) -> pumas_library::Result<Value> {
    let file_path: PathBuf = validate_existing_local_path(
        require_str_param(params, "file_path", "filePath")?,
        "file_path",
    )
    .await?;
    // Count hard links to a file
    if path_exists(&file_path).await? {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if let Ok(metadata) = tokio::fs::metadata(&file_path).await {
                return Ok(json!({
                    "success": true,
                    "count": metadata.nlink()
                }));
            }
        }
    }
    Ok(json!({
        "success": true,
        "count": 1
    }))
}

pub async fn check_files_writable(
    _state: &AppState,
    params: &Value,
) -> pumas_library::Result<Value> {
    // Check if files can be written/modified
    let file_paths: Vec<String> = params
        .get("file_paths")
        .or_else(|| params.get("filePaths"))
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let mut results = Vec::with_capacity(file_paths.len());
    for file_path in &file_paths {
        let writable = if let Ok(path) =
            validate_local_write_target_path(file_path.to_string(), "file_paths").await
        {
            if path_exists(&path).await? {
                tokio::fs::metadata(&path)
                    .await
                    .map(|metadata| !metadata.permissions().readonly())
                    .unwrap_or(false)
            } else if let Some(parent) = path.parent() {
                tokio::fs::metadata(parent)
                    .await
                    .map(|metadata| !metadata.permissions().readonly())
                    .unwrap_or(false)
            } else {
                false
            }
        } else {
            false
        };

        results.push(json!({
            "path": file_path,
            "writable": writable
        }));
    }

    Ok(json!({
        "success": true,
        "results": results
    }))
}

pub async fn set_model_link_exclusion(
    state: &AppState,
    params: &Value,
) -> pumas_library::Result<Value> {
    let model_id = require_str_param(params, "model_id", "modelId")?;
    let app_id = require_str_param(params, "app_id", "appId")?;
    let excluded = params
        .get("excluded")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let response = state
        .api
        .set_model_link_exclusion(&model_id, &app_id, excluded)?;
    Ok(serde_json::to_value(response)?)
}

pub async fn get_link_exclusions(state: &AppState, params: &Value) -> pumas_library::Result<Value> {
    let app_id = require_str_param(params, "app_id", "appId")?;
    let response = state.api.get_link_exclusions(&app_id)?;
    Ok(serde_json::to_value(response)?)
}
