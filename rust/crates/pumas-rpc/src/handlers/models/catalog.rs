//! Model catalog and mapping handlers.

use crate::handlers::require_str_param;
use crate::server::AppState;
use serde_json::{json, Value};

pub async fn get_models(state: &AppState, _params: &Value) -> pumas_library::Result<Value> {
    let models = state.api.list_models().await?;
    // Convert to a format with model_id as keys for frontend compatibility
    let mut result = serde_json::Map::new();
    for model in models {
        result.insert(model.id.clone(), serde_json::to_value(&model)?);
    }
    Ok(json!(result))
}

pub async fn refresh_model_index(
    state: &AppState,
    _params: &Value,
) -> pumas_library::Result<Value> {
    let count = state.api.rebuild_model_index().await?;
    Ok(json!({
        "success": true,
        "indexed_count": count
    }))
}

pub async fn scan_shared_storage(
    state: &AppState,
    _params: &Value,
) -> pumas_library::Result<Value> {
    // Rebuild the model index from metadata files on disk
    let count = state.api.rebuild_model_index().await?;
    Ok(json!({
        "modelsFound": count,
        "scanned": count,
        "indexed": count
    }))
}

pub async fn refetch_model_metadata_from_hf(
    state: &AppState,
    params: &Value,
) -> pumas_library::Result<Value> {
    let model_id = require_str_param(params, "model_id", "modelId")?;

    let updated = state.api.refetch_metadata_from_hf(&model_id).await?;
    Ok(json!({
        "success": true,
        "model_id": model_id,
        "metadata": serde_json::to_value(&updated)?
    }))
}
