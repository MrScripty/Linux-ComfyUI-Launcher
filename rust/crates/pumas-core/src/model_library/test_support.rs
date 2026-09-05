//! Explicit integration-fixture support, absent from default production builds.
//!
//! This adapter exercises current configured-root admission. It does not import
//! legacy stores, bypass the durable publisher, or start network workers.
//! Owned blocking fixtures exercise shutdown through the real HF task owner.

use super::download_recovery::DownloadDestinationRoot;
use super::download_store::{
    DownloadAdmissionDomain, DownloadAdmissionRequest, DownloadPersistence,
};
use crate::models::DownloadStatus;
use crate::{PumasError, Result};
use std::path::Path;

pub use super::download_store::PersistedDownload;

/// Run isolated fixture work through the real download invocation/effect owner.
///
/// The returned future owns its client reference so the API can move into its
/// real server supervisor. The fixture owns every file and synchronization gate
/// used by `work`; this helper does not grant live-library mutation authority.
pub fn run_download_blocking_fixture<T, F>(
    api: &crate::PumasApi,
    work: F,
) -> impl std::future::Future<Output = Result<T>> + Send + 'static
where
    T: Send + 'static,
    F: FnOnce() -> Result<T> + Send + 'static,
{
    let client = api.primary().hf_client.clone();
    async move {
        let client = client.ok_or_else(|| PumasError::Config {
            message: "Download fixture requires an HF client".into(),
        })?;
        client
            .run_download_invocation(move |context| async move {
                context
                    .run_fallible_blocking_named("download integration fixture", work)
                    .await
                    .map_err(|error| {
                        PumasError::Other(format!("Download fixture observation failed: {error}"))
                    })?
            })
            .await
    }
}

/// Admit a paused fixture through the real current-format store owner.
///
/// The caller owns an isolated launcher root and supplies all material snapshot
/// fields. This synchronous setup performs filesystem I/O; invoke it before
/// starting the process under test, not from a production async request.
pub fn admit_paused_download(launcher_root: &Path, snapshot: &PersistedDownload) -> Result<()> {
    if snapshot.status != DownloadStatus::Paused {
        return Err(PumasError::Validation {
            field: "fixture.status".into(),
            message: "Admission fixtures must be paused".into(),
        });
    }
    let root = DownloadDestinationRoot::open(&launcher_root.join("shared-resources/models"))?;
    let destination = root.resolve(&snapshot.dest_dir)?;
    let request = DownloadAdmissionRequest {
        snapshot: snapshot.clone(),
        domain: DownloadAdmissionDomain::Ambient,
        destination: destination.persisted_identity()?,
        requested_payload_files: snapshot.download_request.filenames.clone().ok_or_else(|| {
            PumasError::Validation {
                field: "fixture.download_request.filenames".into(),
                message: "Admission fixtures require explicit payload files".into(),
            }
        })?,
        execution_files: snapshot.filenames.clone(),
    };
    DownloadPersistence::new(&launcher_root.join("launcher-data"))
        .admit_download(&uuid::Uuid::new_v4().to_string(), &request)?
        .into_result()?;
    Ok(())
}
