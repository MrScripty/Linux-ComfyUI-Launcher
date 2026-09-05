//! Shared IPC protocol types and framing.
//!
//! Defines the wire format for local IPC: 4-byte big-endian length prefix
//! followed by a UTF-8 JSON-RPC 2.0 payload.
//!
//! ```text
//! [u32 BE: len][UTF-8 JSON bytes of len]
//! ```

use crate::config::RegistryConfig;
use crate::models::{
    ModelExecutionDescriptorBatchItem, ModelInferenceSettingsBatchItem,
    ModelLibrarySelectorSnapshot, ModelLibrarySelectorSnapshotRequest,
    ModelPackageFactsSummaryBatchItem, ResolveModelArtifactLoadTargetRequest,
    ResolveModelArtifactLoadTargetResponse,
};
use crate::{PumasError, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
#[cfg(test)]
use std::io::{Read, Write};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const MAX_LOCAL_BATCH_ITEMS: usize = 512;
const MAX_LOCAL_STRING_BYTES: usize = 4 * 1024;
const MAX_SELECTOR_LIMIT: u32 = 1_000;

/// Closed set of operations exposed to same-device Pumas clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LocalIpcOperation {
    ModelLibrarySelectorSnapshot,
    ResolveModelArtifactLoadTarget,
    ResolveModelPackageFactsSummaries,
    ResolveModelExecutionDescriptorsBatch,
    GetInferenceSettingsBatch,
    SubscribeModelLibraryUpdateStreamSince,
}

impl LocalIpcOperation {
    pub(crate) fn from_wire_name(name: &str) -> Option<Self> {
        match name {
            "model_library_selector_snapshot" => Some(Self::ModelLibrarySelectorSnapshot),
            "resolve_model_artifact_load_target" => Some(Self::ResolveModelArtifactLoadTarget),
            "resolve_model_package_facts_summaries" => {
                Some(Self::ResolveModelPackageFactsSummaries)
            }
            "resolve_model_execution_descriptors_batch" => {
                Some(Self::ResolveModelExecutionDescriptorsBatch)
            }
            "get_inference_settings_batch" => Some(Self::GetInferenceSettingsBatch),
            "subscribe_model_library_update_stream_since" => {
                Some(Self::SubscribeModelLibraryUpdateStreamSince)
            }
            _ => None,
        }
    }

    pub(crate) fn wire_name(self) -> &'static str {
        match self {
            Self::ModelLibrarySelectorSnapshot => "model_library_selector_snapshot",
            Self::ResolveModelArtifactLoadTarget => "resolve_model_artifact_load_target",
            Self::ResolveModelPackageFactsSummaries => "resolve_model_package_facts_summaries",
            Self::ResolveModelExecutionDescriptorsBatch => {
                "resolve_model_execution_descriptors_batch"
            }
            Self::GetInferenceSettingsBatch => "get_inference_settings_batch",
            Self::SubscribeModelLibraryUpdateStreamSince => {
                "subscribe_model_library_update_stream_since"
            }
        }
    }

    pub(crate) fn validate_outcome(self, value: Value) -> std::result::Result<Value, IpcError> {
        match self {
            Self::ModelLibrarySelectorSnapshot => {
                validate_typed_outcome::<ModelLibrarySelectorSnapshot>(value)
            }
            Self::ResolveModelArtifactLoadTarget => {
                let mut outcome: ResolveModelArtifactLoadTargetResponse =
                    serde_json::from_value(value).map_err(|_| IpcError::internal())?;
                for diagnostic in &mut outcome.diagnostics {
                    diagnostic.field_path = None;
                    diagnostic.message = "Artifact load target is not available".to_string();
                }
                serde_json::to_value(outcome).map_err(|_| IpcError::internal())
            }
            Self::ResolveModelPackageFactsSummaries => {
                let mut outcome: Vec<ModelPackageFactsSummaryBatchItem> =
                    serde_json::from_value(value).map_err(|_| IpcError::internal())?;
                for item in &mut outcome {
                    if item.error.is_some() {
                        item.error = Some("Model package facts unavailable".to_string());
                    }
                }
                serde_json::to_value(outcome).map_err(|_| IpcError::internal())
            }
            Self::ResolveModelExecutionDescriptorsBatch => {
                let mut outcome: Vec<ModelExecutionDescriptorBatchItem> =
                    serde_json::from_value(value).map_err(|_| IpcError::internal())?;
                for item in &mut outcome {
                    if item.error.is_some() {
                        item.error = Some("Model execution descriptor unavailable".to_string());
                    }
                }
                serde_json::to_value(outcome).map_err(|_| IpcError::internal())
            }
            Self::GetInferenceSettingsBatch => {
                let mut outcome: Vec<ModelInferenceSettingsBatchItem> =
                    serde_json::from_value(value).map_err(|_| IpcError::internal())?;
                for item in &mut outcome {
                    if item.error.is_some() {
                        item.error = Some("Model inference settings unavailable".to_string());
                    }
                }
                serde_json::to_value(outcome).map_err(|_| IpcError::internal())
            }
            Self::SubscribeModelLibraryUpdateStreamSince => Err(IpcError::internal()),
        }
    }
}

fn validate_typed_outcome<T>(value: Value) -> std::result::Result<Value, IpcError>
where
    T: serde::de::DeserializeOwned + Serialize,
{
    let outcome: T = serde_json::from_value(value).map_err(|_| IpcError::internal())?;
    serde_json::to_value(outcome).map_err(|_| IpcError::internal())
}

/// A fully decoded command. Its credential is intentionally not `Debug`.
pub(crate) enum LocalIpcCommand {
    ModelLibrarySelectorSnapshot {
        request: ModelLibrarySelectorSnapshotRequest,
        connection_token: String,
    },
    ResolveModelArtifactLoadTarget {
        request: ResolveModelArtifactLoadTargetRequest,
        connection_token: String,
    },
    ResolveModelPackageFactsSummaries {
        model_ids: Vec<String>,
        connection_token: String,
    },
    ResolveModelExecutionDescriptorsBatch {
        model_ids: Vec<String>,
        connection_token: String,
    },
    GetInferenceSettingsBatch {
        model_ids: Vec<String>,
        connection_token: String,
    },
    SubscribeModelLibraryUpdateStreamSince {
        cursor: String,
        connection_token: String,
    },
}

impl LocalIpcCommand {
    pub(crate) fn decode(
        operation: LocalIpcOperation,
        params: Option<Value>,
    ) -> std::result::Result<Self, IpcError> {
        let params = params.ok_or_else(IpcError::invalid_params)?;
        let object = exact_object(
            &params,
            match operation {
                LocalIpcOperation::ModelLibrarySelectorSnapshot
                | LocalIpcOperation::ResolveModelArtifactLoadTarget => {
                    &["request", "connection_token"]
                }
                LocalIpcOperation::ResolveModelPackageFactsSummaries
                | LocalIpcOperation::ResolveModelExecutionDescriptorsBatch
                | LocalIpcOperation::GetInferenceSettingsBatch => {
                    &["model_ids", "connection_token"]
                }
                LocalIpcOperation::SubscribeModelLibraryUpdateStreamSince => {
                    &["cursor", "connection_token"]
                }
            },
        )?;
        let connection_token = required_bounded_string(object, "connection_token")?;

        match operation {
            LocalIpcOperation::ModelLibrarySelectorSnapshot => {
                let request_value = object.get("request").ok_or_else(IpcError::invalid_params)?;
                validate_selector_request(request_value)?;
                let request: ModelLibrarySelectorSnapshotRequest =
                    serde_json::from_value(request_value.clone())
                        .map_err(|_| IpcError::invalid_params())?;
                if request.limit == Some(0) || request.limit.is_some_and(|v| v > MAX_SELECTOR_LIMIT)
                {
                    return Err(IpcError::invalid_params());
                }
                Ok(Self::ModelLibrarySelectorSnapshot {
                    request,
                    connection_token,
                })
            }
            LocalIpcOperation::ResolveModelArtifactLoadTarget => {
                let request_value = object.get("request").ok_or_else(IpcError::invalid_params)?;
                validate_artifact_request(request_value)?;
                let request = serde_json::from_value(request_value.clone())
                    .map_err(|_| IpcError::invalid_params())?;
                Ok(Self::ResolveModelArtifactLoadTarget {
                    request,
                    connection_token,
                })
            }
            LocalIpcOperation::ResolveModelPackageFactsSummaries => {
                Ok(Self::ResolveModelPackageFactsSummaries {
                    model_ids: bounded_model_ids(object)?,
                    connection_token,
                })
            }
            LocalIpcOperation::ResolveModelExecutionDescriptorsBatch => {
                Ok(Self::ResolveModelExecutionDescriptorsBatch {
                    model_ids: bounded_model_ids(object)?,
                    connection_token,
                })
            }
            LocalIpcOperation::GetInferenceSettingsBatch => Ok(Self::GetInferenceSettingsBatch {
                model_ids: bounded_model_ids(object)?,
                connection_token,
            }),
            LocalIpcOperation::SubscribeModelLibraryUpdateStreamSince => {
                Ok(Self::SubscribeModelLibraryUpdateStreamSince {
                    cursor: required_bounded_string(object, "cursor")?,
                    connection_token,
                })
            }
        }
    }

    pub(crate) fn operation(&self) -> LocalIpcOperation {
        match self {
            Self::ModelLibrarySelectorSnapshot { .. } => {
                LocalIpcOperation::ModelLibrarySelectorSnapshot
            }
            Self::ResolveModelArtifactLoadTarget { .. } => {
                LocalIpcOperation::ResolveModelArtifactLoadTarget
            }
            Self::ResolveModelPackageFactsSummaries { .. } => {
                LocalIpcOperation::ResolveModelPackageFactsSummaries
            }
            Self::ResolveModelExecutionDescriptorsBatch { .. } => {
                LocalIpcOperation::ResolveModelExecutionDescriptorsBatch
            }
            Self::GetInferenceSettingsBatch { .. } => LocalIpcOperation::GetInferenceSettingsBatch,
            Self::SubscribeModelLibraryUpdateStreamSince { .. } => {
                LocalIpcOperation::SubscribeModelLibraryUpdateStreamSince
            }
        }
    }

    pub(crate) fn into_dispatch_params(self) -> Value {
        match self {
            Self::ModelLibrarySelectorSnapshot {
                request,
                connection_token,
            } => serde_json::json!({
                "request": request,
                "connection_token": connection_token,
            }),
            Self::ResolveModelArtifactLoadTarget {
                request,
                connection_token,
            } => serde_json::json!({
                "request": request,
                "connection_token": connection_token,
            }),
            Self::ResolveModelPackageFactsSummaries {
                model_ids,
                connection_token,
            }
            | Self::ResolveModelExecutionDescriptorsBatch {
                model_ids,
                connection_token,
            }
            | Self::GetInferenceSettingsBatch {
                model_ids,
                connection_token,
            } => serde_json::json!({
                "model_ids": model_ids,
                "connection_token": connection_token,
            }),
            Self::SubscribeModelLibraryUpdateStreamSince {
                cursor,
                connection_token,
            } => serde_json::json!({
                "cursor": cursor,
                "connection_token": connection_token,
            }),
        }
    }
}

fn exact_object<'a>(
    value: &'a Value,
    allowed_fields: &[&str],
) -> std::result::Result<&'a Map<String, Value>, IpcError> {
    let object = value.as_object().ok_or_else(IpcError::invalid_params)?;
    if object
        .keys()
        .any(|key| !allowed_fields.contains(&key.as_str()))
    {
        return Err(IpcError::invalid_params());
    }
    Ok(object)
}

fn required_bounded_string(
    object: &Map<String, Value>,
    field: &str,
) -> std::result::Result<String, IpcError> {
    let value = object
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty() && value.len() <= MAX_LOCAL_STRING_BYTES)
        .ok_or_else(IpcError::invalid_params)?;
    Ok(value.to_string())
}

fn bounded_model_ids(object: &Map<String, Value>) -> std::result::Result<Vec<String>, IpcError> {
    let values = object
        .get("model_ids")
        .and_then(Value::as_array)
        .filter(|values| !values.is_empty() && values.len() <= MAX_LOCAL_BATCH_ITEMS)
        .ok_or_else(IpcError::invalid_params)?;
    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .filter(|value| !value.trim().is_empty() && value.len() <= MAX_LOCAL_STRING_BYTES)
                .map(str::to_string)
                .ok_or_else(IpcError::invalid_params)
        })
        .collect()
}

fn validate_selector_request(value: &Value) -> std::result::Result<(), IpcError> {
    let object = exact_object(
        value,
        &[
            "offset",
            "limit",
            "search",
            "model_type",
            "task_type_primary",
        ],
    )?;
    for field in ["offset", "limit"] {
        if let Some(value) = object.get(field) {
            if value.is_null() {
                continue;
            }
            let Some(number) = value.as_u64() else {
                return Err(IpcError::invalid_params());
            };
            if number > u32::MAX as u64 {
                return Err(IpcError::invalid_params());
            }
        }
    }
    for field in ["search", "model_type", "task_type_primary"] {
        if let Some(value) = object.get(field) {
            if value.is_null() {
                continue;
            }
            if value.as_str().is_none()
                || value
                    .as_str()
                    .is_some_and(|value| value.len() > MAX_LOCAL_STRING_BYTES)
            {
                return Err(IpcError::invalid_params());
            }
        }
    }
    Ok(())
}

fn validate_artifact_request(value: &Value) -> std::result::Result<(), IpcError> {
    let object = exact_object(
        value,
        &[
            "model_ref",
            "expected_artifact_kind",
            "caller_observed_entry_path",
            "caller_observed_package_facts_contract_version",
            "resolution_mode",
            "consumer",
        ],
    )?;
    let model_ref = object
        .get("model_ref")
        .ok_or_else(IpcError::invalid_params)?;
    exact_object(
        model_ref,
        &[
            "model_ref_contract_version",
            "model_id",
            "revision",
            "selected_artifact_id",
            "selected_artifact_path",
            "migration_diagnostics",
        ],
    )?;
    let consumer = object
        .get("consumer")
        .ok_or_else(IpcError::invalid_params)?;
    exact_object(consumer, &["consumer_name", "task_kind", "runtime_family"])?;
    Ok(())
}

/// JSON-RPC 2.0 request for IPC.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Option<serde_json::Value>,
    pub id: Option<serde_json::Value>,
}

impl IpcRequest {
    /// Create a new JSON-RPC 2.0 request.
    pub(crate) fn new(operation: LocalIpcOperation, params: serde_json::Value, id: u64) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: operation.wire_name().to_string(),
            params: Some(params),
            id: Some(serde_json::Value::Number(id.into())),
        }
    }

    #[cfg(test)]
    pub(crate) fn new_unchecked(
        method: impl Into<String>,
        params: serde_json::Value,
        id: u64,
    ) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params: Some(params),
            id: Some(serde_json::Value::Number(id.into())),
        }
    }
}

/// JSON-RPC 2.0 response for IPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<IpcError>,
    pub id: Option<serde_json::Value>,
}

impl IpcResponse {
    /// Create a success response.
    pub fn success(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    /// Create an error response.
    pub(crate) fn error(id: Option<serde_json::Value>, error: IpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(error),
            id,
        }
    }

    pub(crate) fn into_result(self, expected_id: u64) -> Result<Value> {
        let expected_id = Some(Value::Number(expected_id.into()));
        if self.jsonrpc != "2.0" || self.id != expected_id {
            return Err(PumasError::InvalidParams {
                message: "Invalid local IPC response correlation".to_string(),
            });
        }

        match (self.result, self.error) {
            (Some(result), None) => Ok(result),
            (None, Some(error)) if error.code == -32602 => Err(PumasError::InvalidParams {
                message: error.message,
            }),
            (None, Some(error)) => Err(PumasError::Other(error.message)),
            _ => Err(PumasError::InvalidParams {
                message: "Invalid local IPC response outcome".to_string(),
            }),
        }
    }
}

/// JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl IpcError {
    pub(crate) fn parse_error() -> Self {
        Self::public(-32700, "parse_error", "Malformed local IPC JSON")
    }

    pub(crate) fn invalid_request() -> Self {
        Self::public(-32600, "invalid_request", "Invalid local IPC request")
    }

    pub(crate) fn method_not_found() -> Self {
        Self::public(
            -32601,
            "method_not_found",
            "Unsupported local IPC operation",
        )
    }

    pub(crate) fn invalid_params() -> Self {
        Self::public(-32602, "invalid_params", "Invalid local IPC parameters")
    }

    pub(crate) fn internal() -> Self {
        Self::public(-32603, "internal", "Local IPC operation failed")
    }

    pub(crate) fn from_pumas(error: &PumasError) -> Self {
        let code = error.to_rpc_error_code();
        let class = match code {
            -32602 | -32005 => "invalid_params",
            -32000 | -32006 | -32012 => "unavailable",
            -32001 | -32002 | -32009 | -32010 => "not_found",
            -32004 => "cancelled",
            -32011 => "conflict",
            -32603 => "internal",
            _ => "operation_failed",
        };
        let message = match class {
            "invalid_params" => "Invalid local IPC parameters",
            "unavailable" => "Local IPC service unavailable",
            "not_found" => "Requested local resource not found",
            "cancelled" => "Local IPC operation cancelled",
            "conflict" => "Local IPC operation conflicted with current state",
            "internal" => "Local IPC operation failed",
            _ => "Local IPC operation could not be completed",
        };
        Self::public(code, class, message)
    }

    fn public(code: i32, class: &'static str, message: &'static str) -> Self {
        Self {
            code,
            message: message.to_string(),
            data: Some(serde_json::json!({ "class": class })),
        }
    }
}

/// Read a length-prefixed frame from an async reader.
///
/// Frame format: `[4-byte BE u32 length][payload bytes]`
///
/// Returns `None` on clean EOF (peer closed connection).
pub async fn read_frame<R: AsyncReadExt + Unpin>(reader: &mut R) -> Result<Option<Vec<u8>>> {
    let mut len_buf = [0u8; 4];
    match reader.read_exact(&mut len_buf).await {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e.into()),
    }

    let len = u32::from_be_bytes(len_buf) as usize;

    if len > RegistryConfig::MAX_IPC_MESSAGE_SIZE {
        return Err(PumasError::Validation {
            field: "ipc_frame".to_string(),
            message: format!(
                "IPC message size {} exceeds maximum {}",
                len,
                RegistryConfig::MAX_IPC_MESSAGE_SIZE
            ),
        });
    }

    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload).await?;

    Ok(Some(payload))
}

/// Write a length-prefixed frame to an async writer.
///
/// Frame format: `[4-byte BE u32 length][payload bytes]`
pub async fn write_frame<W: AsyncWriteExt + Unpin>(writer: &mut W, payload: &[u8]) -> Result<()> {
    let len = payload.len() as u32;
    writer.write_all(&len.to_be_bytes()).await?;
    writer.write_all(payload).await?;
    writer.flush().await?;
    Ok(())
}

/// Read a length-prefixed frame from a blocking reader.
#[cfg(test)]
pub(crate) fn read_frame_blocking<R: Read>(reader: &mut R) -> Result<Option<Vec<u8>>> {
    let mut len_buf = [0u8; 4];
    match reader.read_exact(&mut len_buf) {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e.into()),
    }

    let len = u32::from_be_bytes(len_buf) as usize;

    if len > RegistryConfig::MAX_IPC_MESSAGE_SIZE {
        return Err(PumasError::Validation {
            field: "ipc_frame".to_string(),
            message: format!(
                "IPC message size {} exceeds maximum {}",
                len,
                RegistryConfig::MAX_IPC_MESSAGE_SIZE
            ),
        });
    }

    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload)?;

    Ok(Some(payload))
}

/// Write a length-prefixed frame to a blocking writer.
#[cfg(test)]
pub(crate) fn write_frame_blocking<W: Write>(writer: &mut W, payload: &[u8]) -> Result<()> {
    let len = payload.len() as u32;
    writer.write_all(&len.to_be_bytes())?;
    writer.write_all(payload)?;
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_root_busy_preserves_existing_ipc_conflict_projection() {
        let error = IpcError::from_pumas(&PumasError::DownloadRootBusy);
        let wire = serde_json::to_value(&error).unwrap();
        assert_eq!(
            wire,
            serde_json::json!({
                "code": -32011,
                "message": "Local IPC operation conflicted with current state",
                "data": { "class": "conflict" },
            })
        );
        let decoded: IpcError = serde_json::from_value(wire).unwrap();
        assert_eq!(decoded.code, -32011);
        assert_eq!(
            decoded.data,
            Some(serde_json::json!({ "class": "conflict" }))
        );
        let response = IpcResponse::error(Some(serde_json::json!(1)), decoded);
        assert!(matches!(
            response.into_result(1),
            Err(PumasError::Other(message))
                if message == "Local IPC operation conflicted with current state"
        ));
    }

    #[test]
    fn test_ipc_request_serialization_roundtrip() {
        let req = IpcRequest::new(
            LocalIpcOperation::ModelLibrarySelectorSnapshot,
            serde_json::json!({
                "request": { "limit": 25 },
                "connection_token": "test-token",
            }),
            1,
        );
        let json = serde_json::to_string(&req).unwrap();
        let parsed: IpcRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.jsonrpc, "2.0");
        assert_eq!(parsed.method, "model_library_selector_snapshot");
        assert_eq!(parsed.id, Some(serde_json::Value::Number(1.into())));
    }

    #[test]
    fn local_ipc_command_rejects_missing_null_extra_negative_and_oversized_values() {
        let operation = LocalIpcOperation::ModelLibrarySelectorSnapshot;
        for params in [
            None,
            Some(Value::Null),
            Some(serde_json::json!({
                "request": {},
                "connection_token": "test-token",
                "extra": true,
            })),
            Some(serde_json::json!({
                "request": { "offset": -1 },
                "connection_token": "test-token",
            })),
            Some(serde_json::json!({
                "request": { "limit": 1001 },
                "connection_token": "test-token",
            })),
        ] {
            let error = match LocalIpcCommand::decode(operation, params) {
                Ok(_) => panic!("invalid local IPC params were accepted"),
                Err(error) => error,
            };
            assert_eq!(error.code, -32602);
            assert_eq!(
                error.data,
                Some(serde_json::json!({ "class": "invalid_params" }))
            );
        }
    }

    #[test]
    fn local_ipc_command_rejects_unknown_nested_fields_and_unbounded_batches() {
        let artifact = serde_json::json!({
            "request": {
                "model_ref": {
                    "model_id": "repo/model",
                    "unexpected": true,
                },
                "resolution_mode": "owner_fresh",
                "consumer": { "consumer_name": "test" },
            },
            "connection_token": "test-token",
        });
        assert!(LocalIpcCommand::decode(
            LocalIpcOperation::ResolveModelArtifactLoadTarget,
            Some(artifact),
        )
        .is_err());

        let model_ids = vec!["model"; MAX_LOCAL_BATCH_ITEMS + 1];
        assert!(LocalIpcCommand::decode(
            LocalIpcOperation::GetInferenceSettingsBatch,
            Some(serde_json::json!({
                "model_ids": model_ids,
                "connection_token": "test-token",
            })),
        )
        .is_err());
    }

    #[test]
    fn local_ipc_outcome_rejects_wrong_type() {
        let error = LocalIpcOperation::ModelLibrarySelectorSnapshot
            .validate_outcome(serde_json::json!({ "success": true }))
            .unwrap_err();
        assert_eq!(error.code, -32603);
    }

    #[test]
    fn local_ipc_response_rejects_wrong_version_id_and_ambiguous_outcome() {
        let mut wrong_version = IpcResponse::success(Some(Value::from(1)), Value::Null);
        wrong_version.jsonrpc = "1.0".to_string();
        assert!(matches!(
            wrong_version.into_result(1),
            Err(PumasError::InvalidParams { .. })
        ));

        let wrong_id = IpcResponse::success(Some(Value::from(2)), Value::Null);
        assert!(matches!(
            wrong_id.into_result(1),
            Err(PumasError::InvalidParams { .. })
        ));

        let ambiguous = IpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(Value::Null),
            error: Some(IpcError::internal()),
            id: Some(Value::from(1)),
        };
        assert!(matches!(
            ambiguous.into_result(1),
            Err(PumasError::InvalidParams { .. })
        ));
    }

    #[test]
    fn test_ipc_response_success_serialization() {
        let resp = IpcResponse::success(
            Some(serde_json::Value::Number(1.into())),
            serde_json::json!({"models": []}),
        );
        let json = serde_json::to_string(&resp).unwrap();

        assert!(json.contains("\"result\""));
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn test_ipc_response_error_serialization() {
        let resp = IpcResponse::error(
            Some(serde_json::Value::Number(1.into())),
            IpcError::internal(),
        );
        let json = serde_json::to_string(&resp).unwrap();

        assert!(!json.contains("\"result\""));
        assert!(json.contains("\"error\""));
        assert!(json.contains("-32603"));
    }

    #[tokio::test]
    async fn test_frame_read_write_roundtrip() {
        let payload = b"hello world";
        let mut buf = Vec::new();

        write_frame(&mut buf, payload).await.unwrap();

        let mut cursor = std::io::Cursor::new(buf);
        let read_back = read_frame(&mut cursor).await.unwrap();

        assert_eq!(read_back, Some(payload.to_vec()));
    }

    #[tokio::test]
    async fn test_frame_read_empty_stream_returns_none() {
        let mut cursor = std::io::Cursor::new(Vec::<u8>::new());
        let result = read_frame(&mut cursor).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_frame_read_oversized_returns_error() {
        // Craft a frame header claiming a huge payload
        let huge_len: u32 = (RegistryConfig::MAX_IPC_MESSAGE_SIZE + 1) as u32;
        let mut buf = Vec::new();
        buf.extend_from_slice(&huge_len.to_be_bytes());
        buf.extend_from_slice(&[0u8; 8]); // some bytes but not enough

        let mut cursor = std::io::Cursor::new(buf);
        let result = read_frame(&mut cursor).await;
        assert!(result.is_err());
    }
}
