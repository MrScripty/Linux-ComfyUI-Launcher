//! Model download handlers.

use crate::contract::{
    DownloadListOutcome, DownloadMutationOutcome, DownloadStartedOutcome, DownloadStatusOutcome,
    PartialDownloadOutcome,
};
use crate::server::AppState;
use pumas_library::model_library::{DownloadRecoveryModelId, DownloadRecoveryToken};
use pumas_library::DownloadRequest;

pub async fn download_model_from_hf(
    state: &AppState,
    request: DownloadRequest,
) -> pumas_library::Result<DownloadStartedOutcome> {
    start_download(state, request).await
}

pub async fn start_model_download_from_hf(
    state: &AppState,
    request: DownloadRequest,
) -> pumas_library::Result<DownloadStartedOutcome> {
    start_download(state, request).await
}

async fn start_download(
    state: &AppState,
    request: DownloadRequest,
) -> pumas_library::Result<DownloadStartedOutcome> {
    match state.api.start_hf_download(&request).await {
        Ok(download_id) => download_started(state, download_id).await,
        Err(error) => Ok(DownloadStartedOutcome::failed(&error)),
    }
}

async fn download_started(
    state: &AppState,
    download_id: String,
) -> pumas_library::Result<DownloadStartedOutcome> {
    let selected_artifact_id = state
        .api
        .get_hf_download_progress(&download_id)
        .await?
        .and_then(|progress| progress.selected_artifact_id);
    Ok(DownloadStartedOutcome::started(
        download_id,
        selected_artifact_id,
    ))
}

pub async fn get_model_download_status(
    state: &AppState,
    download_id: &str,
) -> pumas_library::Result<DownloadStatusOutcome> {
    DownloadStatusOutcome::new(state.api.get_hf_download_progress(download_id).await?)
}

pub async fn cancel_model_download(
    state: &AppState,
    download_id: &str,
) -> pumas_library::Result<DownloadMutationOutcome> {
    Ok(DownloadMutationOutcome::completed(
        state.api.cancel_hf_download(download_id).await?,
    ))
}

pub async fn pause_model_download(
    state: &AppState,
    download_id: &str,
) -> pumas_library::Result<DownloadMutationOutcome> {
    Ok(DownloadMutationOutcome::completed(
        state.api.pause_hf_download(download_id).await?,
    ))
}

pub async fn resume_model_download(
    state: &AppState,
    download_id: &str,
) -> pumas_library::Result<DownloadMutationOutcome> {
    Ok(DownloadMutationOutcome::completed(
        state.api.resume_hf_download(download_id).await?,
    ))
}

pub async fn list_model_downloads(state: &AppState) -> pumas_library::Result<DownloadListOutcome> {
    DownloadListOutcome::new(state.api.list_hf_downloads().await?)
}

pub async fn resume_partial_download(
    state: &AppState,
    model_id: &DownloadRecoveryModelId,
    recovery_token: &DownloadRecoveryToken,
) -> pumas_library::Result<PartialDownloadOutcome> {
    state
        .api
        .resume_partial_download_with_ticket(model_id, recovery_token)
        .await?
        .try_into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_start_response_includes_selected_artifact_aliases() {
        let response = serde_json::to_value(DownloadStartedOutcome::started(
            "dl-1".to_string(),
            Some("owner--model__q4_k_m".to_string()),
        ))
        .unwrap();

        assert_eq!(response["success"], true);
        assert_eq!(response["download_id"], "dl-1");
        assert_eq!(response["selectedArtifactId"], "owner--model__q4_k_m");
        assert_eq!(response["artifactId"], "owner--model__q4_k_m");
    }
}
