//! Explicit integration-fixture support, absent from default production builds.
//!
//! This adapter exercises current configured-root admission. It does not import
//! legacy stores, bypass the durable publisher, or start network workers.

use super::download_recovery::DownloadDestinationRoot;
use super::download_store::{
    DownloadAdmissionDomain, DownloadAdmissionRequest, DownloadPersistence,
};
use crate::models::DownloadStatus;
use crate::{PumasError, Result};
use std::path::Path;

pub use super::download_store::PersistedDownload;

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
