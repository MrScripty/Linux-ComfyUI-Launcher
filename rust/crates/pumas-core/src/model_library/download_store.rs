//! Download persistence for crash recovery and restart resume.
//!
//! The versioned JSON store retains resumable snapshots, exact-attempt admission
//! and release records, recovery revocations, cleanup quarantines, and pending
//! relocation intents. Strict inventory hides unresolved ownership transitions;
//! durable terminal proofs may outlive the resumable snapshot they protect.

use crate::error::Result;
use crate::metadata::{
    AtomicJsonTarget, AtomicPublication, AtomicPublishFailure, AtomicPublishFailureKind,
    AtomicPublishResult, AtomicPublishStage, StagingCleanup,
};
use crate::model_library::types::DownloadRequest;
use crate::models::DownloadStatus;
use crate::models::HuggingFaceEvidence;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use tracing::{debug, warn};
use uuid::Uuid;

const DOWNLOAD_STORE_SCHEMA_VERSION: u32 = 3;
const DOWNLOAD_STORE_LOCK_FILE: &str = ".downloads.lock";

/// A single persisted download entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedDownload {
    pub download_id: String,
    pub repo_id: String,
    /// Primary filename (first file or legacy single-file download).
    pub filename: String,
    /// All filenames in this download (for multi-file models).
    /// Empty means legacy single-file download (use `filename` field).
    #[serde(default)]
    pub filenames: Vec<String>,
    pub dest_dir: PathBuf,
    pub total_bytes: Option<u64>,
    pub status: DownloadStatus,
    pub download_request: DownloadRequest,
    pub created_at: String,
    /// Known SHA256 from HuggingFace LFS metadata (avoids recomputation on import).
    #[serde(default)]
    pub known_sha256: Option<String>,
    /// Normalized HuggingFace evidence captured during download preflight.
    #[serde(default)]
    pub huggingface_evidence: Option<HuggingFaceEvidence>,
}

/// Persisted ownership domain for a quarantined download lifecycle.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LifecycleQuarantineDomain {
    Ambient,
    Recovery,
}

/// Cleanup proof exposed to the HF lifecycle owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LifecycleCleanupDisposition {
    Pending,
    Verified,
}

#[derive(Debug, Clone)]
pub(crate) struct LifecycleQuarantine {
    pub(crate) snapshot: PersistedDownload,
    pub(crate) domain: LifecycleQuarantineDomain,
    pub(crate) disposition: LifecycleCleanupDisposition,
    pub(crate) sticky_failure: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct PersistedDownloadInventory {
    pub(crate) downloads: Vec<PersistedDownload>,
    pub(crate) quarantines: BTreeMap<String, LifecycleQuarantine>,
    pub(crate) hidden_admissions: BTreeMap<String, HiddenDownloadAdmission>,
    pub(crate) queue_admissions: BTreeMap<String, PersistedQueueAdmission>,
    pub(crate) pending_relocations: BTreeMap<String, PendingLegacyRelocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct LegacyRelocationRequest {
    pub(crate) source: PersistedDestinationIdentity,
    pub(crate) target: PersistedDestinationIdentity,
    pub(crate) target_dir: PathBuf,
    pub(crate) model_type: Option<String>,
    pub(crate) family: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingLegacyRelocation {
    pub(crate) request: LegacyRelocationRequest,
    pub(crate) snapshot: PersistedDownload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedLegacyRelocation {
    attempt_id: String,
    request: LegacyRelocationRequest,
}

/// Non-authorizing equality identity for one destination below the configured
/// model-library root. Runtime filesystem authority is held separately.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub(crate) struct PersistedDestinationIdentity {
    pub(crate) library_root: String,
    pub(crate) relative_target: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DownloadAdmissionDomain {
    Ambient,
    Recovery,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DownloadAdmissionRequest {
    pub(crate) snapshot: PersistedDownload,
    pub(crate) domain: DownloadAdmissionDomain,
    pub(crate) destination: PersistedDestinationIdentity,
    pub(crate) requested_payload_files: Vec<String>,
    pub(crate) execution_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct QueuePredecessor {
    pub(crate) download_id: String,
    pub(crate) admission_attempt_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct DownloadAdmissionPosition {
    pub(crate) ordinal: u64,
    pub(crate) predecessor: Option<QueuePredecessor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PersistedQueueAdmission {
    pub(crate) attempt_id: String,
    pub(crate) domain: DownloadAdmissionDomain,
    pub(crate) destination: PersistedDestinationIdentity,
    pub(crate) requested_payload_files: Vec<String>,
    pub(crate) execution_files: Vec<String>,
    pub(crate) position: DownloadAdmissionPosition,
}

#[derive(Debug, Clone)]
pub(crate) struct HiddenDownloadAdmission {
    pub(crate) request: DownloadAdmissionRequest,
    pub(crate) position: DownloadAdmissionPosition,
}

#[derive(Debug, Clone)]
pub(crate) struct DurableDownloadAdmission {
    pub(crate) position: DownloadAdmissionPosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DownloadAdmissionPhase {
    Intent,
    Confirmation,
}

#[derive(Debug)]
pub(crate) enum DownloadAdmissionTransition {
    Durable {
        admission: DurableDownloadAdmission,
    },
    NotPublished {
        attempt_id: String,
        phase: DownloadAdmissionPhase,
        stage: AtomicPublishStage,
        kind: AtomicPublishFailureKind,
        error: crate::PumasError,
        cleanup: StagingCleanup,
    },
    PublishedDurabilityUnknown {
        attempt_id: String,
        phase: DownloadAdmissionPhase,
        error: crate::PumasError,
    },
    VisibilityUnknown {
        attempt_id: String,
        phase: DownloadAdmissionPhase,
        error: crate::PumasError,
        cleanup: StagingCleanup,
    },
}

impl DownloadAdmissionTransition {
    /// Record failed-operation causality while preserving the underlying typed
    /// error for callers that classify failures by variant or source.
    pub(crate) fn into_result(self) -> Result<DurableDownloadAdmission> {
        let error = match self {
            Self::Durable { admission } => return Ok(admission),
            Self::NotPublished {
                attempt_id,
                phase,
                stage,
                kind,
                error,
                cleanup,
            } => {
                let error = admission_error_context(error, &format!(
                    "Download admission {attempt_id} {phase:?} was not published at {stage:?} ({kind:?})"
                ));
                AtomicPublishFailure { stage, kind, error, cleanup }.into_error()
            }
            Self::PublishedDurabilityUnknown {
                attempt_id,
                phase,
                error,
            } => {
                admission_error_context(error, &format!(
                    "Download admission {attempt_id} {phase:?} has unknown durability; effects remain blocked"
                ))
            }
            Self::VisibilityUnknown {
                attempt_id,
                phase,
                error,
                cleanup,
            } => {
                let error = admission_error_context(error, &format!(
                    "Download admission {attempt_id} {phase:?} has unknown visibility; effects remain blocked"
                ));
                match cleanup {
                    StagingCleanup::Failed { error: cleanup } => crate::PumasError::Other(format!(
                        "{error}; staging cleanup also failed: {cleanup}"
                    )),
                    StagingCleanup::NotRequired | StagingCleanup::Removed => error,
                }
            }
        };
        Err(error)
    }
}

/// Atomic publication emits these contextual variants. Preserve any other
/// injected error unchanged rather than destroying its classification.
fn admission_error_context(mut error: crate::PumasError, context: &str) -> crate::PumasError {
    match &mut error {
        crate::PumasError::Io { message, .. }
        | crate::PumasError::Json { message, .. }
        | crate::PumasError::Validation { message, .. }
        | crate::PumasError::Other(message) => {
            *message = format!("{context}: {message}");
        }
        _ => {}
    }
    error
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedAdmissionAttempt {
    request: DownloadAdmissionRequest,
    position: DownloadAdmissionPosition,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum PersistedLifecycleCleanupDisposition {
    PendingIntent,
    Pending,
    VerifiedIntent,
    Verified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedLifecycleQuarantine {
    snapshot: PersistedDownload,
    domain: LifecycleQuarantineDomain,
    disposition: PersistedLifecycleCleanupDisposition,
    sticky_failure: bool,
}

/// Current downloads.json document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DownloadStoreData {
    schema_version: u32,
    downloads: Vec<PersistedDownload>,
    recovery_revocations: BTreeMap<String, PersistedRecoveryRevocation>,
    lifecycle_quarantines: BTreeMap<String, PersistedLifecycleQuarantine>,
    admission_attempts: BTreeMap<String, PersistedAdmissionAttempt>,
    queue_admissions: BTreeMap<String, PersistedQueueAdmission>,
    #[serde(default)]
    released_queue_admissions: BTreeMap<String, PersistedQueueAdmission>,
    #[serde(default)]
    pending_relocations: BTreeMap<String, PersistedLegacyRelocation>,
}

impl DownloadStoreData {
    fn empty() -> Self {
        Self {
            schema_version: DOWNLOAD_STORE_SCHEMA_VERSION,
            downloads: Vec::new(),
            recovery_revocations: BTreeMap::new(),
            lifecycle_quarantines: BTreeMap::new(),
            admission_attempts: BTreeMap::new(),
            queue_admissions: BTreeMap::new(),
            released_queue_admissions: BTreeMap::new(),
            pending_relocations: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionOneDownloadStoreData {
    schema_version: u32,
    downloads: Vec<PersistedDownload>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionTwoDownloadStoreData {
    schema_version: u32,
    downloads: Vec<PersistedDownload>,
    recovery_revocations: BTreeMap<String, PersistedRecoveryRevocation>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyDownloadStoreData {
    downloads: Vec<PersistedDownload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedRecoveryRevocation {
    attempt_id: String,
    disposition: PersistedRevocationDisposition,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum PersistedRevocationDisposition {
    DurabilityUnknown,
    Durable,
}

/// Manages download persistence to `downloads.json`.
#[derive(Clone)]
pub struct DownloadPersistence {
    path: PathBuf,
    mutation: Arc<Mutex<()>>,
    confirmed_admissions: Arc<Mutex<HashSet<String>>>,
    confirmed_cleanups: Arc<Mutex<HashSet<String>>>,
    publisher: Arc<dyn DownloadStorePublisher>,
    observer: Arc<dyn StoreTransactionObserver>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StoreOperation {
    Load,
    Admit,
    Save,
    Remove,
    UpdateStatus,
    Relocate,
    Revoke,
}

trait DownloadStorePublisher: Send + Sync {
    fn publish(&self, target: &AtomicJsonTarget, data: &DownloadStoreData) -> AtomicPublishResult;
}

struct AtomicDownloadStorePublisher;

impl DownloadStorePublisher for AtomicDownloadStorePublisher {
    fn publish(&self, target: &AtomicJsonTarget, data: &DownloadStoreData) -> AtomicPublishResult {
        target.publish_json(data)
    }
}

trait StoreTransactionObserver: Send + Sync {
    fn attempting(&self, _operation: StoreOperation) {}

    fn acquired(&self, _operation: StoreOperation) {}
}

struct NoopStoreTransactionObserver;

impl StoreTransactionObserver for NoopStoreTransactionObserver {}

struct StoreTransaction<'a> {
    _instance_guard: MutexGuard<'a, ()>,
    target: AtomicJsonTarget,
    _os_lock: File,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecoveryRevocationPhase {
    Intent,
    Confirmation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecoveryRevocationSource {
    NewlyPublished,
    Persisted,
}

/// Result of publishing removal of ambient recovery authority.
#[derive(Debug)]
pub(crate) enum RecoveryRevocation {
    Durable {
        source: RecoveryRevocationSource,
        attempt_id: String,
    },
    NotPublished {
        phase: RecoveryRevocationPhase,
        stage: AtomicPublishStage,
        kind: AtomicPublishFailureKind,
        error: crate::PumasError,
        cleanup: StagingCleanup,
    },
    PublishedDurabilityUnknown {
        phase: RecoveryRevocationPhase,
        error: crate::PumasError,
    },
    VisibilityUnknown {
        phase: RecoveryRevocationPhase,
        error: crate::PumasError,
        cleanup: StagingCleanup,
    },
}

impl RecoveryRevocation {
    fn into_legacy_result(self) -> Result<()> {
        match self {
            Self::Durable { source, attempt_id } => {
                debug!("Recovery revocation {attempt_id} is durable ({source:?})");
                Ok(())
            }
            Self::NotPublished {
                phase,
                stage,
                kind,
                error,
                cleanup,
            } => {
                let error = AtomicPublishFailure {
                    stage,
                    kind,
                    error,
                    cleanup,
                }
                .into_error();
                Err(crate::PumasError::Other(format!(
                    "Recovery revocation {phase:?} publication failed: {error}"
                )))
            }
            Self::VisibilityUnknown {
                phase,
                error,
                cleanup,
            } => {
                let error = AtomicPublishFailure {
                    stage: AtomicPublishStage::Rename,
                    kind: AtomicPublishFailureKind::Filesystem,
                    error,
                    cleanup,
                }
                .into_error();
                Err(crate::PumasError::Other(format!(
                    "Recovery revocation {phase:?} publication failed: {error}"
                )))
            }
            Self::PublishedDurabilityUnknown { phase, error } => Err(crate::PumasError::Other(
                format!("Recovery revocation {phase:?} durability is unknown: {error}"),
            )),
        }
    }
}

impl DownloadPersistence {
    /// Create a new persistence store at `{data_dir}/downloads.json`.
    pub fn new(data_dir: &Path) -> Self {
        Self {
            path: data_dir.join("downloads.json"),
            mutation: Arc::new(Mutex::new(())),
            confirmed_admissions: Arc::new(Mutex::new(HashSet::new())),
            confirmed_cleanups: Arc::new(Mutex::new(HashSet::new())),
            publisher: Arc::new(AtomicDownloadStorePublisher),
            observer: Arc::new(NoopStoreTransactionObserver),
        }
    }

    #[cfg(test)]
    fn with_test_publisher(mut self, publisher: Arc<dyn DownloadStorePublisher>) -> Self {
        self.publisher = publisher;
        self
    }

    #[cfg(test)]
    fn with_test_observer(mut self, observer: Arc<dyn StoreTransactionObserver>) -> Self {
        self.observer = observer;
        self
    }

    fn confirmed_admission_ids(&self) -> Result<HashSet<String>> {
        self.confirmed_admissions
            .lock()
            .map(|ids| ids.clone())
            .map_err(|_| {
                crate::PumasError::Other(
                    "Download admission confirmation lock is poisoned".to_string(),
                )
            })
    }

    fn confirm_admission(&self, attempt_id: &str) -> Result<()> {
        self.confirmed_admissions
            .lock()
            .map_err(|_| {
                crate::PumasError::Other(
                    "Download admission confirmation lock is poisoned".to_string(),
                )
            })?
            .insert(attempt_id.to_string());
        Ok(())
    }

    /// Establish a new durability barrier before restoring promoted admissions.
    /// Reading identical bytes cannot reveal whether the preceding process saw
    /// a successful directory sync. Fresh owners therefore keep admissions
    /// hidden until this publication succeeds. Intent-only attempts remain
    /// hidden and require an exact `admit_download` retry.
    pub(crate) fn reconcile_lifecycle_inventory_strict(&self) -> Result<()> {
        let transaction = self.transaction(StoreOperation::Admit)?;
        let mut data = self.load_data_strict(&transaction)?;
        self.write_data(&transaction, &mut data)?;
        for admission in data.queue_admissions.values() {
            self.confirm_admission(&admission.attempt_id)?;
        }
        let mut confirmed = self.confirmed_cleanups.lock().map_err(|_| {
            crate::PumasError::Other("Download cleanup confirmation lock is poisoned".into())
        })?;
        confirmed.extend(
            data.lifecycle_quarantines
                .iter()
                .filter(|(_, quarantine)| {
                    quarantine.disposition == PersistedLifecycleCleanupDisposition::Verified
                })
                .map(|(id, _)| id.clone()),
        );
        Ok(())
    }

    /// Settle exactly one admission after its runtime owner has completed all
    /// destination effects. Retain the immutable queue record as predecessor
    /// proof; absence is never proof of a successful earlier release.
    /// Sticky quarantine provenance survives settlement.
    pub(crate) fn settle_queue_admission(
        &self,
        download_id: &str,
        attempt_id: &str,
    ) -> Result<bool> {
        let transaction = self.transaction(StoreOperation::Remove)?;
        let mut data = self.load_data_strict(&transaction)?;
        let Some(admission) = data
            .queue_admissions
            .get(download_id)
            .or_else(|| data.released_queue_admissions.get(download_id))
        else {
            return Ok(false);
        };
        if admission.attempt_id != attempt_id {
            return Err(crate::PumasError::Validation {
                field: "downloads.queue_admissions".into(),
                message: "Queue release attempt identity mismatch".into(),
            });
        }
        if let Some(quarantine) = data.lifecycle_quarantines.get(download_id) {
            if quarantine.sticky_failure
                && quarantine.disposition != PersistedLifecycleCleanupDisposition::Verified
            {
                return Err(crate::PumasError::Validation {
                    field: "downloads.lifecycle_quarantines".into(),
                    message: "Queue release requires verified failure cleanup".into(),
                });
            }
        }
        if let Some(admission) = data.queue_admissions.remove(download_id) {
            data.released_queue_admissions
                .insert(download_id.to_string(), admission);
            data.downloads
                .retain(|download| download.download_id != download_id);
            data.lifecycle_quarantines
                .retain(|id, quarantine| id != download_id || quarantine.sticky_failure);
        }
        self.write_data(&transaction, &mut data)?;
        Ok(true)
    }

    /// Upsert a download entry (insert or update by download_id).
    pub fn save(&self, download: &PersistedDownload) -> Result<()> {
        let transaction = self.transaction(StoreOperation::Save)?;
        let mut data = self.load_data_strict(&transaction)?;
        reject_queue_mutation(&data, &download.download_id)?;
        reject_pending_relocation_path(&data, &download.dest_dir)?;
        if data
            .recovery_revocations
            .contains_key(&download.download_id)
            || data
                .lifecycle_quarantines
                .contains_key(&download.download_id)
            || data
                .released_queue_admissions
                .contains_key(&download.download_id)
        {
            return Err(crate::PumasError::Other(
                "Refusing to persist a revoked or lifecycle-quarantined download".to_string(),
            ));
        }
        if let Some(existing) = data
            .downloads
            .iter_mut()
            .find(|d| d.download_id == download.download_id)
        {
            *existing = download.clone();
        } else {
            data.downloads.push(download.clone());
        }
        self.write_data(&transaction, &mut data)
    }

    /// Remove a download entry by ID.
    pub fn remove(&self, download_id: &str) -> Result<()> {
        let transaction = self.transaction(StoreOperation::Remove)?;
        let mut data = self.load_data_strict(&transaction)?;
        reject_queue_mutation(&data, download_id)?;
        if data.recovery_revocations.contains_key(download_id)
            || data.lifecycle_quarantines.contains_key(download_id)
        {
            return Ok(());
        }
        let before = data.downloads.len();
        data.downloads.retain(|d| d.download_id != download_id);
        if data.downloads.len() < before {
            self.write_data(&transaction, &mut data)?;
        }
        Ok(())
    }

    /// Load all persisted downloads.
    pub fn load_all(&self) -> Vec<PersistedDownload> {
        match self.load_all_strict() {
            Ok(downloads) => downloads,
            Err(error) => {
                warn!(
                    "Failed to read download store at {}: {}",
                    self.path.display(),
                    error
                );
                Vec::new()
            }
        }
    }

    pub(crate) fn load_all_strict(&self) -> Result<Vec<PersistedDownload>> {
        let transaction = self.transaction(StoreOperation::Load)?;
        let mut data = self.load_data_strict(&transaction)?;
        let confirmed = self.confirmed_admission_ids()?;
        data.downloads.retain(|download| {
            !data.pending_relocations.contains_key(&download.download_id)
                && data
                    .queue_admissions
                    .get(&download.download_id)
                    .is_none_or(|admission| confirmed.contains(&admission.attempt_id))
        });
        Ok(data.downloads)
    }

    pub(crate) fn load_lifecycle_inventory_strict(&self) -> Result<PersistedDownloadInventory> {
        let transaction = self.transaction(StoreOperation::Load)?;
        let mut data = self.load_data_strict(&transaction)?;
        let confirmed = self.confirmed_admission_ids()?;
        let pending_relocations = data
            .pending_relocations
            .iter()
            .map(|(id, relocation)| {
                let snapshot = data
                    .downloads
                    .iter()
                    .find(|snapshot| snapshot.download_id == *id)
                    .expect("validated relocation retains its source row")
                    .clone();
                (
                    id.clone(),
                    PendingLegacyRelocation {
                        request: relocation.request.clone(),
                        snapshot,
                    },
                )
            })
            .collect();
        let mut hidden_admissions: BTreeMap<String, HiddenDownloadAdmission> = data
            .admission_attempts
            .values()
            .map(|attempt| {
                (
                    attempt.request.snapshot.download_id.clone(),
                    HiddenDownloadAdmission {
                        request: attempt.request.clone(),
                        position: attempt.position.clone(),
                    },
                )
            })
            .collect();
        let unconfirmed_ids: Vec<String> = data
            .queue_admissions
            .iter()
            .filter(|(_, admission)| !confirmed.contains(&admission.attempt_id))
            .map(|(download_id, _)| download_id.clone())
            .collect();
        for download_id in &unconfirmed_ids {
            let admission = data
                .queue_admissions
                .get(download_id)
                .expect("unconfirmed queue admission was collected above");
            let snapshot = data
                .downloads
                .iter()
                .find(|snapshot| snapshot.download_id == *download_id)
                .or_else(|| {
                    data.lifecycle_quarantines
                        .get(download_id)
                        .map(|quarantine| &quarantine.snapshot)
                })
                .expect("validated queue admission has a snapshot owner")
                .clone();
            hidden_admissions.insert(
                download_id.clone(),
                HiddenDownloadAdmission {
                    request: DownloadAdmissionRequest {
                        snapshot,
                        domain: admission.domain,
                        destination: admission.destination.clone(),
                        requested_payload_files: admission.requested_payload_files.clone(),
                        execution_files: admission.execution_files.clone(),
                    },
                    position: admission.position.clone(),
                },
            );
        }
        data.downloads.retain(|download| {
            !data.pending_relocations.contains_key(&download.download_id)
                && !unconfirmed_ids
                    .iter()
                    .any(|download_id| download_id == &download.download_id)
        });
        data.queue_admissions
            .retain(|_, admission| confirmed.contains(&admission.attempt_id));
        let confirmed_cleanups = self.confirmed_cleanups.lock().map_err(|_| {
            crate::PumasError::Other("Download cleanup confirmation lock is poisoned".into())
        })?;
        Ok(PersistedDownloadInventory {
            pending_relocations,
            downloads: data.downloads,
            quarantines: data
                .lifecycle_quarantines
                .into_iter()
                .map(|(download_id, quarantine)| {
                    (
                        download_id.clone(),
                        LifecycleQuarantine {
                            snapshot: quarantine.snapshot,
                            domain: quarantine.domain,
                            disposition: if quarantine.disposition
                                == PersistedLifecycleCleanupDisposition::Verified
                                && confirmed_cleanups.contains(&download_id)
                            {
                                LifecycleCleanupDisposition::Verified
                            } else {
                                LifecycleCleanupDisposition::Pending
                            },
                            sticky_failure: quarantine.sticky_failure,
                        },
                    )
                })
                .collect(),
            hidden_admissions,
            queue_admissions: data.queue_admissions,
        })
    }

    /// Persist an admission intent and then atomically promote its immutable
    /// snapshot into the public ordinary-row and durable FIFO inventory.
    ///
    /// The caller chooses and retains `attempt_id`. Any non-durable outcome is
    /// fail closed: a matching retry must republish the same phase to a
    /// confirmed barrier before it may use the admission.
    pub(crate) fn admit_download(
        &self,
        attempt_id: &str,
        request: &DownloadAdmissionRequest,
    ) -> Result<DownloadAdmissionTransition> {
        Uuid::parse_str(attempt_id).map_err(|source| crate::PumasError::Validation {
            field: "downloads.admission_attempts".to_string(),
            message: format!("Invalid download admission attempt: {source}"),
        })?;
        validate_admission_request(request)?;

        let transaction = self.transaction(StoreOperation::Admit)?;
        let mut data = self.load_data_strict(&transaction)?;
        let download_id = request.snapshot.download_id.as_str();
        reject_pending_relocation(&data, download_id)?;
        if data.pending_relocations.values().any(|relocation| {
            relocation.request.source == request.destination
                || relocation.request.target == request.destination
        }) {
            return Err(relocation_error(
                "Destination is reserved by a pending relocation",
            ));
        }
        if data.recovery_revocations.contains_key(download_id)
            || data.lifecycle_quarantines.contains_key(download_id)
            || data.released_queue_admissions.contains_key(download_id)
        {
            return Err(crate::PumasError::Other(
                "Refusing to admit a revoked or lifecycle-quarantined download".to_string(),
            ));
        }

        if let Some(admission) = data.queue_admissions.get(download_id) {
            if admission.attempt_id != attempt_id
                || !queue_admission_matches_request(admission, request)
                || !data.downloads.iter().any(|snapshot| {
                    snapshot.download_id == download_id
                        && persisted_download_matches(snapshot, &request.snapshot)
                })
            {
                return Err(crate::PumasError::Other(
                    "Download admission identity mismatch".to_string(),
                ));
            }
            let position = admission.position.clone();
            validate_store_data(&data)?;
            let publication = self.publisher.publish(&transaction.target, &data);
            if let Some(outcome) = admission_publication_outcome(
                attempt_id,
                DownloadAdmissionPhase::Confirmation,
                publication,
            ) {
                return Ok(outcome);
            }
            self.confirm_admission(attempt_id)?;
            return Ok(DownloadAdmissionTransition::Durable {
                admission: DurableDownloadAdmission { position },
            });
        }

        if data
            .queue_admissions
            .values()
            .chain(data.released_queue_admissions.values())
            .any(|admission| admission.attempt_id == attempt_id)
        {
            return Err(crate::PumasError::Other(
                "Download admission attempt belongs to another download".to_string(),
            ));
        }
        if data
            .downloads
            .iter()
            .any(|snapshot| snapshot.download_id == download_id)
        {
            return Err(crate::PumasError::Other(
                "Download ID already has an ordinary persisted owner".to_string(),
            ));
        }

        let position = match data.admission_attempts.get(attempt_id) {
            Some(existing) => {
                if !admission_request_matches(&existing.request, request) {
                    return Err(crate::PumasError::Other(
                        "Download admission attempt identity mismatch".to_string(),
                    ));
                }
                existing.position.clone()
            }
            None => {
                if data.admission_attempts.values().any(|attempt| {
                    attempt.request.snapshot.download_id == request.snapshot.download_id
                }) {
                    return Err(crate::PumasError::Other(
                        "Download ID already has a hidden admission owner".to_string(),
                    ));
                }
                let position = next_admission_position(&data, request)?;
                data.admission_attempts.insert(
                    attempt_id.to_string(),
                    PersistedAdmissionAttempt {
                        request: request.clone(),
                        position: position.clone(),
                    },
                );
                position
            }
        };

        // Always republish the exact intent, including after an ambiguous
        // prior call. Presence in a strict reread is not a durability proof.
        validate_store_data(&data)?;
        let intent = self.publisher.publish(&transaction.target, &data);
        if let Some(outcome) =
            admission_publication_outcome(attempt_id, DownloadAdmissionPhase::Intent, intent)
        {
            return Ok(outcome);
        }

        data.admission_attempts.remove(attempt_id);
        data.downloads.push(request.snapshot.clone());
        data.queue_admissions.insert(
            request.snapshot.download_id.clone(),
            PersistedQueueAdmission {
                attempt_id: attempt_id.to_string(),
                domain: request.domain,
                destination: request.destination.clone(),
                requested_payload_files: request.requested_payload_files.clone(),
                execution_files: request.execution_files.clone(),
                position: position.clone(),
            },
        );
        validate_store_data(&data)?;
        let confirmation = self.publisher.publish(&transaction.target, &data);
        if let Some(outcome) = admission_publication_outcome(
            attempt_id,
            DownloadAdmissionPhase::Confirmation,
            confirmation,
        ) {
            return Ok(outcome);
        }
        self.confirm_admission(attempt_id)?;
        Ok(DownloadAdmissionTransition::Durable {
            admission: DurableDownloadAdmission { position },
        })
    }

    /// Atomically remove ordinary resumable state and durably publish a
    /// fail-closed Pending quarantine before cleanup is attempted.
    pub(crate) fn begin_lifecycle_quarantine(
        &self,
        snapshot: &PersistedDownload,
        domain: LifecycleQuarantineDomain,
        sticky_failure: bool,
    ) -> Result<()> {
        let transaction = self.transaction(StoreOperation::UpdateStatus)?;
        let mut data = self.load_data_strict(&transaction)?;
        let download_id = snapshot.download_id.clone();
        reject_pending_relocation(&data, &download_id)?;
        reject_pending_relocation_path(&data, &snapshot.dest_dir)?;
        if let Some(existing) = data
            .downloads
            .iter()
            .find(|entry| entry.download_id == download_id)
        {
            if data.queue_admissions.contains_key(&download_id)
                && !persisted_download_identity_matches(existing, snapshot)
            {
                return Err(crate::PumasError::Validation {
                    field: "downloads.lifecycle_quarantines".into(),
                    message: "Quarantine must preserve the admitted snapshot identity".into(),
                });
            }
        }
        match domain {
            LifecycleQuarantineDomain::Ambient => {
                if data.recovery_revocations.contains_key(&download_id) {
                    return Err(crate::PumasError::Other(
                        "Ambient lifecycle quarantine conflicts with recovery revocation"
                            .to_string(),
                    ));
                }
            }
            LifecycleQuarantineDomain::Recovery => {
                let Some(revocation) = data.recovery_revocations.get(&download_id) else {
                    return Err(crate::PumasError::Other(
                        "Recovery lifecycle quarantine requires a revocation tombstone".to_string(),
                    ));
                };
                if revocation.disposition != PersistedRevocationDisposition::Durable {
                    return Err(crate::PumasError::Other(
                        "Recovery lifecycle quarantine requires a durable revocation tombstone"
                            .to_string(),
                    ));
                }
            }
        }

        let sticky_failure = sticky_failure
            || data
                .lifecycle_quarantines
                .get(&download_id)
                .is_some_and(|existing| existing.sticky_failure);
        let mut quarantine_snapshot = snapshot.clone();
        quarantine_snapshot.status = if sticky_failure {
            DownloadStatus::Error
        } else {
            DownloadStatus::Cancelling
        };
        if data.lifecycle_quarantines.contains_key(&download_id) {
            let (disposition, promote_failure) = {
                let existing = data
                    .lifecycle_quarantines
                    .get_mut(&download_id)
                    .expect("quarantine presence checked above");
                if existing.domain != domain
                    || !persisted_download_identity_matches(
                        &existing.snapshot,
                        &quarantine_snapshot,
                    )
                {
                    return Err(crate::PumasError::Other(
                        "Lifecycle quarantine identity mismatch".to_string(),
                    ));
                }
                let promote_failure = sticky_failure && !existing.sticky_failure;
                if promote_failure {
                    existing.sticky_failure = true;
                    existing.snapshot.status = DownloadStatus::Error;
                }
                (existing.disposition, promote_failure)
            };
            if promote_failure {
                self.write_data(&transaction, &mut data)?;
            }
            if matches!(
                disposition,
                PersistedLifecycleCleanupDisposition::Pending
                    | PersistedLifecycleCleanupDisposition::Verified
                    | PersistedLifecycleCleanupDisposition::VerifiedIntent
            ) {
                return self.write_data(&transaction, &mut data);
            }
        }

        data.downloads
            .retain(|download| download.download_id != download_id);
        data.lifecycle_quarantines.insert(
            download_id.clone(),
            PersistedLifecycleQuarantine {
                snapshot: quarantine_snapshot,
                domain,
                disposition: PersistedLifecycleCleanupDisposition::PendingIntent,
                sticky_failure,
            },
        );
        self.write_data(&transaction, &mut data)?;
        data.lifecycle_quarantines
            .get_mut(&download_id)
            .expect("quarantine was inserted above")
            .disposition = PersistedLifecycleCleanupDisposition::Pending;
        self.write_data(&transaction, &mut data)
    }

    /// Confirm cleanup through a two-publication transition. An ambiguous
    /// publication never authorizes the current runtime to release custody.
    pub(crate) fn verify_lifecycle_quarantine(&self, download_id: &str) -> Result<bool> {
        let transaction = self.transaction(StoreOperation::UpdateStatus)?;
        let mut data = self.load_data_strict(&transaction)?;
        let Some(quarantine) = data.lifecycle_quarantines.get_mut(download_id) else {
            return Ok(false);
        };
        if !quarantine.sticky_failure {
            return Err(crate::PumasError::Other(
                "Clean lifecycle quarantine cannot be verified as sticky failure".to_string(),
            ));
        }
        match quarantine.disposition {
            PersistedLifecycleCleanupDisposition::Verified => {}
            PersistedLifecycleCleanupDisposition::PendingIntent => return Ok(false),
            PersistedLifecycleCleanupDisposition::Pending => {
                quarantine.disposition = PersistedLifecycleCleanupDisposition::VerifiedIntent;
                self.write_data(&transaction, &mut data)?;
            }
            PersistedLifecycleCleanupDisposition::VerifiedIntent => {}
        }
        data.lifecycle_quarantines
            .get_mut(download_id)
            .expect("quarantine remains present")
            .disposition = PersistedLifecycleCleanupDisposition::Verified;
        self.write_data(&transaction, &mut data)?;
        self.confirmed_cleanups
            .lock()
            .map_err(|_| {
                crate::PumasError::Other("Download cleanup confirmation lock is poisoned".into())
            })?
            .insert(download_id.to_string());
        Ok(true)
    }

    pub(crate) fn mark_lifecycle_quarantine_failed(&self, download_id: &str) -> Result<bool> {
        let transaction = self.transaction(StoreOperation::UpdateStatus)?;
        let mut data = self.load_data_strict(&transaction)?;
        let Some(quarantine) = data.lifecycle_quarantines.get_mut(download_id) else {
            return Ok(false);
        };
        if quarantine.sticky_failure {
            self.write_data(&transaction, &mut data)?;
            return Ok(true);
        }
        quarantine.sticky_failure = true;
        quarantine.snapshot.status = DownloadStatus::Error;
        self.write_data(&transaction, &mut data)?;
        Ok(true)
    }

    pub(crate) fn remove_clean_lifecycle_quarantine(&self, download_id: &str) -> Result<bool> {
        let transaction = self.transaction(StoreOperation::Remove)?;
        let mut data = self.load_data_strict(&transaction)?;
        reject_queue_mutation(&data, download_id)?;
        let Some(quarantine) = data.lifecycle_quarantines.get(download_id) else {
            return Ok(false);
        };
        if quarantine.sticky_failure {
            return Err(crate::PumasError::Other(
                "Refusing to remove lifecycle failure provenance as clean cancellation".to_string(),
            ));
        }
        if quarantine.disposition != PersistedLifecycleCleanupDisposition::Pending {
            return Ok(false);
        }
        data.lifecycle_quarantines.remove(download_id);
        self.write_data(&transaction, &mut data)?;
        Ok(true)
    }

    /// Remove a download through the versioned publication Interface and
    /// prevent stale writers from recreating its ambient destination authority.
    pub(crate) fn revoke(&self, download_id: &str) -> Result<()> {
        self.revoke_for_recovery(download_id)?.into_legacy_result()
    }

    /// Publish revocation and retain fail-closed state if durability is unknown.
    pub(crate) fn revoke_for_recovery(&self, download_id: &str) -> Result<RecoveryRevocation> {
        let transaction = self.transaction(StoreOperation::Revoke)?;
        let mut data = self.load_data_strict(&transaction)?;
        reject_queue_mutation(&data, download_id)?;
        if let Some(existing) = data.recovery_revocations.get(download_id) {
            if existing.disposition == PersistedRevocationDisposition::Durable {
                return Ok(RecoveryRevocation::Durable {
                    source: RecoveryRevocationSource::Persisted,
                    attempt_id: existing.attempt_id.clone(),
                });
            }
        }

        let attempt_id = Uuid::new_v4().to_string();
        data.downloads
            .retain(|download| download.download_id != download_id);
        data.recovery_revocations.insert(
            download_id.to_string(),
            PersistedRecoveryRevocation {
                attempt_id: attempt_id.clone(),
                disposition: PersistedRevocationDisposition::DurabilityUnknown,
            },
        );
        validate_store_data(&data)?;
        let intent = self.publisher.publish(&transaction.target, &data);
        if let Some(outcome) =
            revocation_publication_outcome(RecoveryRevocationPhase::Intent, intent)
        {
            return Ok(outcome);
        }

        data.recovery_revocations
            .get_mut(download_id)
            .expect("revocation inserted above")
            .disposition = PersistedRevocationDisposition::Durable;
        validate_store_data(&data)?;
        let confirmation = self.publisher.publish(&transaction.target, &data);
        if let Some(outcome) =
            revocation_publication_outcome(RecoveryRevocationPhase::Confirmation, confirmation)
        {
            return Ok(outcome);
        }
        Ok(RecoveryRevocation::Durable {
            source: RecoveryRevocationSource::NewlyPublished,
            attempt_id,
        })
    }

    pub(crate) fn is_revoked(&self, download_id: &str) -> Result<bool> {
        let transaction = self.transaction(StoreOperation::Load)?;
        Ok(self
            .load_data_strict(&transaction)?
            .recovery_revocations
            .contains_key(download_id))
    }

    /// Change only the status of this owner's confirmed, exact admission.
    /// Cancellation and completion require quarantine or queue settlement so
    /// a status write cannot release destination authority.
    pub(crate) fn update_admitted_status(
        &self,
        download_id: &str,
        attempt_id: &str,
        status: DownloadStatus,
    ) -> Result<bool> {
        if matches!(
            status,
            DownloadStatus::Cancelling | DownloadStatus::Completed | DownloadStatus::Cancelled
        ) {
            return Err(crate::PumasError::Validation {
                field: "downloads.status".into(),
                message: "Terminal or cancelling status requires an owned queue settlement".into(),
            });
        }
        let transaction = self.transaction(StoreOperation::UpdateStatus)?;
        let mut data = self.load_data_strict(&transaction)?;
        if data.recovery_revocations.contains_key(download_id)
            || data.lifecycle_quarantines.contains_key(download_id)
            || data
                .queue_admissions
                .get(download_id)
                .is_none_or(|admission| admission.attempt_id != attempt_id)
            || !self.confirmed_admission_ids()?.contains(attempt_id)
        {
            return Ok(false);
        }
        let Some(entry) = data
            .downloads
            .iter_mut()
            .find(|entry| entry.download_id == download_id)
        else {
            return Ok(false);
        };
        entry.status = status;
        self.write_data(&transaction, &mut data)?;
        Ok(true)
    }

    pub(crate) fn update_status(&self, download_id: &str, status: DownloadStatus) -> Result<bool> {
        let transaction = self.transaction(StoreOperation::UpdateStatus)?;
        let mut data = self.load_data_strict(&transaction)?;
        reject_pending_relocation(&data, download_id)?;
        if data.recovery_revocations.contains_key(download_id)
            || data.lifecycle_quarantines.contains_key(download_id)
            || data.queue_admissions.contains_key(download_id)
        {
            return Ok(false);
        }
        let Some(entry) = data
            .downloads
            .iter_mut()
            .find(|entry| entry.download_id == download_id)
        else {
            return Ok(false);
        };
        entry.status = status;
        self.write_data(&transaction, &mut data)?;
        Ok(true)
    }

    /// Park both destinations before the lifecycle owner performs any move.
    /// The retained source row is hidden until finish or a proved-not-moved abort.
    pub(crate) fn begin_legacy_relocation(
        &self,
        attempt_id: &str,
        expected: &PersistedDownload,
        request: &LegacyRelocationRequest,
    ) -> Result<AtomicPublishResult> {
        Uuid::parse_str(attempt_id).map_err(|_| relocation_error("Invalid relocation attempt"))?;
        validate_relocation_request(request)?;
        let transaction = self.transaction(StoreOperation::Relocate)?;
        let mut data = self.load_data_strict(&transaction)?;
        let id = &expected.download_id;
        if data.queue_admissions.contains_key(id)
            || data.released_queue_admissions.contains_key(id)
            || data.recovery_revocations.contains_key(id)
            || data.lifecycle_quarantines.contains_key(id)
            || data
                .admission_attempts
                .values()
                .any(|entry| entry.request.snapshot.download_id == *id)
        {
            return Err(relocation_error(
                "Relocation requires an ordinary legacy owner",
            ));
        }
        let current = data
            .downloads
            .iter()
            .find(|entry| entry.download_id == *id)
            .ok_or_else(|| relocation_error("Relocation source is missing"))?;
        if !persisted_download_matches(current, expected) {
            return Err(relocation_error("Relocation source snapshot changed"));
        }
        if !matches!(
            current.status,
            DownloadStatus::Paused | DownloadStatus::Error
        ) {
            return Err(relocation_error("Relocation source must be inactive"));
        }
        if let Some(existing) = data.pending_relocations.get(id) {
            if existing.attempt_id != attempt_id || existing.request != *request {
                return Err(relocation_error("Relocation attempt identity mismatch"));
            }
        } else {
            if data
                .pending_relocations
                .values()
                .any(|entry| entry.attempt_id == attempt_id)
                || data.admission_attempts.contains_key(attempt_id)
                || data
                    .queue_admissions
                    .values()
                    .chain(data.released_queue_admissions.values())
                    .any(|entry| entry.attempt_id == attempt_id)
            {
                return Err(relocation_error(
                    "Relocation attempt already belongs to another owner",
                ));
            }
            data.pending_relocations.insert(
                id.clone(),
                PersistedLegacyRelocation {
                    attempt_id: attempt_id.to_string(),
                    request: request.clone(),
                },
            );
        }
        validate_store_data(&data)?;
        Ok(self.publisher.publish(&transaction.target, &data))
    }

    /// Called only after the lifecycle owner proves the move, marker, and
    /// affected directory syncs durable. A visible target-only document is
    /// therefore safe to restore even if the publication acknowledgement fails.
    pub(crate) fn finish_legacy_relocation(
        &self,
        download_id: &str,
        attempt_id: &str,
    ) -> Result<AtomicPublishResult> {
        self.complete_legacy_relocation(download_id, attempt_id, true)
    }

    /// Called only when the lifecycle owner proves no physical mutation occurred.
    /// Missing intents are conflicts, not permission to repeat physical effects.
    pub(crate) fn abort_legacy_relocation(
        &self,
        download_id: &str,
        attempt_id: &str,
    ) -> Result<AtomicPublishResult> {
        self.complete_legacy_relocation(download_id, attempt_id, false)
    }

    fn complete_legacy_relocation(
        &self,
        download_id: &str,
        attempt_id: &str,
        moved: bool,
    ) -> Result<AtomicPublishResult> {
        let transaction = self.transaction(StoreOperation::Relocate)?;
        let mut data = self.load_data_strict(&transaction)?;
        let relocation = data.pending_relocations.get(download_id).ok_or_else(|| {
            relocation_error("Relocation intent is missing; inspect authoritative state")
        })?;
        if relocation.attempt_id != attempt_id {
            return Err(relocation_error("Relocation attempt identity mismatch"));
        }
        if moved {
            let row = data
                .downloads
                .iter_mut()
                .find(|row| row.download_id == download_id)
                .expect("validated relocation retains source row");
            row.dest_dir = relocation.request.target_dir.clone();
            if let Some(model_type) = &relocation.request.model_type {
                row.download_request.model_type = Some(model_type.clone());
            }
            if let Some(family) = &relocation.request.family {
                row.download_request.family = family.clone();
            }
        }
        data.pending_relocations.remove(download_id);
        validate_store_data(&data)?;
        Ok(self.publisher.publish(&transaction.target, &data))
    }

    fn transaction(&self, operation: StoreOperation) -> Result<StoreTransaction<'_>> {
        let instance_guard = self.mutation.lock().map_err(|_| {
            crate::PumasError::Other("Download persistence lock is poisoned".to_string())
        })?;
        let target = AtomicJsonTarget::open(&self.path)?;
        let os_lock = target.open_lock_file(DOWNLOAD_STORE_LOCK_FILE)?;
        self.observer.attempting(operation);
        os_lock.lock().map_err(|source| crate::PumasError::Io {
            message: format!("Failed to lock download store {}", self.path.display()),
            path: Some(self.path.clone()),
            source: Some(source),
        })?;
        self.observer.acquired(operation);
        Ok(StoreTransaction {
            _instance_guard: instance_guard,
            target,
            _os_lock: os_lock,
        })
    }

    fn load_data_strict(&self, transaction: &StoreTransaction<'_>) -> Result<DownloadStoreData> {
        let Some(value) = transaction.target.read_json::<serde_json::Value>()? else {
            return Ok(DownloadStoreData::empty());
        };
        let data = match value.get("schema_version") {
            Some(version) => match version.as_u64() {
                Some(3) => {
                    serde_json::from_value::<DownloadStoreData>(value).map_err(|source| {
                        crate::PumasError::Json {
                            message: format!(
                                "Failed to parse versioned download store {}: {source}",
                                self.path.display()
                            ),
                            source: Some(source),
                        }
                    })?
                }
                Some(2) => {
                    let old = serde_json::from_value::<VersionTwoDownloadStoreData>(value)
                        .map_err(|source| crate::PumasError::Json {
                            message: format!(
                                "Failed to parse v2 download store {}: {source}",
                                self.path.display()
                            ),
                            source: Some(source),
                        })?;
                    debug_assert_eq!(old.schema_version, 2);
                    DownloadStoreData {
                        downloads: old.downloads,
                        recovery_revocations: old.recovery_revocations,
                        ..DownloadStoreData::empty()
                    }
                }
                Some(1) => {
                    let old = serde_json::from_value::<VersionOneDownloadStoreData>(value)
                        .map_err(|source| crate::PumasError::Json {
                            message: format!(
                                "Failed to parse v1 download store {}: {source}",
                                self.path.display()
                            ),
                            source: Some(source),
                        })?;
                    debug_assert_eq!(old.schema_version, 1);
                    DownloadStoreData {
                        downloads: old.downloads,
                        ..DownloadStoreData::empty()
                    }
                }
                _ => {
                    return Err(crate::PumasError::Validation {
                        field: "downloads.schema_version".to_string(),
                        message: format!(
                            "Unsupported download store schema version in {}",
                            self.path.display()
                        ),
                    });
                }
            },
            None => {
                let legacy =
                    serde_json::from_value::<LegacyDownloadStoreData>(value).map_err(|source| {
                        crate::PumasError::Json {
                            message: format!(
                                "Failed to parse legacy download store {}: {source}",
                                self.path.display()
                            ),
                            source: Some(source),
                        }
                    })?;
                DownloadStoreData {
                    downloads: legacy.downloads,
                    ..DownloadStoreData::empty()
                }
            }
        };
        validate_store_data(&data)?;
        Ok(data)
    }

    /// Replace the complete versioned store document and require `Durable`.
    fn write_data(
        &self,
        transaction: &StoreTransaction<'_>,
        data: &mut DownloadStoreData,
    ) -> Result<()> {
        validate_store_data(data)?;
        debug!(
            "Writing {} downloads to {}",
            data.downloads.len(),
            self.path.display()
        );
        match self.publisher.publish(&transaction.target, data) {
            Ok(AtomicPublication::Durable) => Ok(()),
            Ok(AtomicPublication::PublishedDurabilityUnknown { error }) => Err(error),
            Ok(AtomicPublication::VisibilityUnknown { error, cleanup }) => {
                Err(AtomicPublishFailure {
                    stage: AtomicPublishStage::Rename,
                    kind: AtomicPublishFailureKind::Filesystem,
                    error,
                    cleanup,
                }
                .into_error())
            }
            Err(failure) => Err((*failure).into_error()),
        }
    }
}

fn reject_queue_mutation(data: &DownloadStoreData, download_id: &str) -> Result<()> {
    reject_pending_relocation(data, download_id)?;
    if data.queue_admissions.contains_key(download_id) {
        return Err(crate::PumasError::Validation {
            field: "downloads.queue_admissions".into(),
            message: "Queue-owned download requires an exact lifecycle transition".into(),
        });
    }
    Ok(())
}

fn relocation_error(message: &str) -> crate::PumasError {
    crate::PumasError::Validation {
        field: "downloads.pending_relocations".into(),
        message: message.into(),
    }
}

fn reject_pending_relocation(data: &DownloadStoreData, download_id: &str) -> Result<()> {
    if data.pending_relocations.contains_key(download_id) {
        return Err(relocation_error(
            "Download is owned by a pending relocation",
        ));
    }
    Ok(())
}

fn validate_relocation_request(request: &LegacyRelocationRequest) -> Result<()> {
    if request.source.library_root.trim().is_empty()
        || request.source.relative_target.trim().is_empty()
        || request.target.relative_target.trim().is_empty()
        || request.source.library_root != request.target.library_root
        || request.source == request.target
        || request.target_dir.as_os_str().is_empty()
    {
        return Err(relocation_error(
            "Relocation requires distinct destinations under the same root",
        ));
    }
    Ok(())
}

fn reject_pending_relocation_path(data: &DownloadStoreData, destination: &Path) -> Result<()> {
    if data.pending_relocations.iter().any(|(id, relocation)| {
        relocation.request.target_dir == destination
            || data
                .downloads
                .iter()
                .any(|row| row.download_id == *id && row.dest_dir == destination)
    }) {
        return Err(relocation_error("Path is reserved by a pending relocation"));
    }
    Ok(())
}

fn revocation_publication_outcome(
    phase: RecoveryRevocationPhase,
    publication: AtomicPublishResult,
) -> Option<RecoveryRevocation> {
    match publication {
        Ok(AtomicPublication::Durable) => None,
        Err(failure) => {
            let AtomicPublishFailure {
                stage,
                kind,
                error,
                cleanup,
            } = *failure;
            Some(RecoveryRevocation::NotPublished {
                phase,
                stage,
                kind,
                error,
                cleanup,
            })
        }
        Ok(AtomicPublication::PublishedDurabilityUnknown { error }) => {
            Some(RecoveryRevocation::PublishedDurabilityUnknown { phase, error })
        }
        Ok(AtomicPublication::VisibilityUnknown { error, cleanup }) => {
            Some(RecoveryRevocation::VisibilityUnknown {
                phase,
                error,
                cleanup,
            })
        }
    }
}

fn admission_publication_outcome(
    attempt_id: &str,
    phase: DownloadAdmissionPhase,
    publication: AtomicPublishResult,
) -> Option<DownloadAdmissionTransition> {
    match publication {
        Ok(AtomicPublication::Durable) => None,
        Err(failure) => {
            let AtomicPublishFailure {
                stage,
                kind,
                error,
                cleanup,
            } = *failure;
            Some(DownloadAdmissionTransition::NotPublished {
                attempt_id: attempt_id.to_string(),
                phase,
                stage,
                kind,
                error,
                cleanup,
            })
        }
        Ok(AtomicPublication::PublishedDurabilityUnknown { error }) => {
            Some(DownloadAdmissionTransition::PublishedDurabilityUnknown {
                attempt_id: attempt_id.to_string(),
                phase,
                error,
            })
        }
        Ok(AtomicPublication::VisibilityUnknown { error, cleanup }) => {
            Some(DownloadAdmissionTransition::VisibilityUnknown {
                attempt_id: attempt_id.to_string(),
                phase,
                error,
                cleanup,
            })
        }
    }
}

fn validate_admission_request(request: &DownloadAdmissionRequest) -> Result<()> {
    if request.snapshot.download_id.trim().is_empty() {
        return Err(crate::PumasError::Validation {
            field: "downloads.admission_attempts".to_string(),
            message: "Download admission has an empty download ID".to_string(),
        });
    }
    if request.destination.library_root.trim().is_empty()
        || request.destination.relative_target.trim().is_empty()
    {
        return Err(crate::PumasError::Validation {
            field: "downloads.admission_attempts".to_string(),
            message: "Download admission has an empty destination identity".to_string(),
        });
    }
    if request.requested_payload_files.is_empty() || request.execution_files.is_empty() {
        return Err(crate::PumasError::Validation {
            field: "downloads.admission_attempts".to_string(),
            message: "Download admission requires payload and execution files".to_string(),
        });
    }
    let mut requested = HashSet::new();
    if request
        .requested_payload_files
        .iter()
        .any(|file| file.trim().is_empty() || !requested.insert(file.as_str()))
    {
        return Err(crate::PumasError::Validation {
            field: "downloads.admission_attempts".to_string(),
            message: "Download admission payload files must be non-empty and unique".to_string(),
        });
    }
    let mut execution = HashSet::new();
    if request
        .execution_files
        .iter()
        .any(|file| file.trim().is_empty() || !execution.insert(file.as_str()))
        || !requested.is_subset(&execution)
    {
        return Err(crate::PumasError::Validation {
            field: "downloads.admission_attempts".to_string(),
            message: "Download admission execution files must be unique and contain the payload"
                .to_string(),
        });
    }
    Ok(())
}

fn persisted_download_matches(left: &PersistedDownload, right: &PersistedDownload) -> bool {
    serde_json::to_value(left).ok() == serde_json::to_value(right).ok()
}

fn persisted_download_identity_matches(
    left: &PersistedDownload,
    right: &PersistedDownload,
) -> bool {
    let mut candidate = right.clone();
    candidate.status = left.status;
    persisted_download_matches(left, &candidate)
}

fn admission_request_matches(
    left: &DownloadAdmissionRequest,
    right: &DownloadAdmissionRequest,
) -> bool {
    left.domain == right.domain
        && left.destination == right.destination
        && left.requested_payload_files == right.requested_payload_files
        && left.execution_files == right.execution_files
        && persisted_download_matches(&left.snapshot, &right.snapshot)
}

fn queue_admission_matches_request(
    admission: &PersistedQueueAdmission,
    request: &DownloadAdmissionRequest,
) -> bool {
    admission.domain == request.domain
        && admission.destination == request.destination
        && admission.requested_payload_files == request.requested_payload_files
        && admission.execution_files == request.execution_files
}

fn next_admission_position(
    data: &DownloadStoreData,
    request: &DownloadAdmissionRequest,
) -> Result<DownloadAdmissionPosition> {
    let mut latest: Option<(u64, QueuePredecessor)> = None;
    for (download_id, admission) in data
        .queue_admissions
        .iter()
        .chain(&data.released_queue_admissions)
    {
        if admission.destination == request.destination {
            let candidate = (
                admission.position.ordinal,
                QueuePredecessor {
                    download_id: download_id.clone(),
                    admission_attempt_id: admission.attempt_id.clone(),
                },
            );
            if latest
                .as_ref()
                .is_none_or(|(ordinal, _)| candidate.0 > *ordinal)
            {
                latest = Some(candidate);
            }
        }
    }
    for (attempt_id, attempt) in &data.admission_attempts {
        if attempt.request.destination == request.destination {
            let candidate = (
                attempt.position.ordinal,
                QueuePredecessor {
                    download_id: attempt.request.snapshot.download_id.clone(),
                    admission_attempt_id: attempt_id.clone(),
                },
            );
            if latest
                .as_ref()
                .is_none_or(|(ordinal, _)| candidate.0 > *ordinal)
            {
                latest = Some(candidate);
            }
        }
    }
    let ordinal = match latest.as_ref() {
        Some((ordinal, _)) => {
            ordinal
                .checked_add(1)
                .ok_or_else(|| crate::PumasError::Validation {
                    field: "downloads.queue_admissions".to_string(),
                    message: "Download admission ordinal overflow".to_string(),
                })?
        }
        None => 0,
    };
    Ok(DownloadAdmissionPosition {
        ordinal,
        predecessor: latest.map(|(_, predecessor)| predecessor),
    })
}

fn validate_store_data(data: &DownloadStoreData) -> Result<()> {
    let mut relocation_destinations = BTreeSet::new();
    let mut relocation_attempts = HashSet::new();
    for (id, relocation) in &data.pending_relocations {
        validate_relocation_request(&relocation.request)?;
        Uuid::parse_str(&relocation.attempt_id)
            .map_err(|_| relocation_error("Invalid relocation attempt"))?;
        if !relocation_attempts.insert(&relocation.attempt_id)
            || data.admission_attempts.contains_key(&relocation.attempt_id)
            || data
                .queue_admissions
                .values()
                .chain(data.released_queue_admissions.values())
                .any(|entry| entry.attempt_id == relocation.attempt_id)
        {
            return Err(relocation_error("Relocation attempt has another owner"));
        }
        if !data.downloads.iter().any(|row| {
            row.download_id == *id
                && matches!(row.status, DownloadStatus::Paused | DownloadStatus::Error)
        }) || data.queue_admissions.contains_key(id)
            || data.released_queue_admissions.contains_key(id)
            || data.recovery_revocations.contains_key(id)
            || data.lifecycle_quarantines.contains_key(id)
            || data
                .admission_attempts
                .values()
                .any(|entry| entry.request.snapshot.download_id == *id)
        {
            return Err(relocation_error(
                "Pending relocation lacks an exclusive inactive legacy row",
            ));
        }
        let source = data
            .downloads
            .iter()
            .find(|row| row.download_id == *id)
            .expect("relocation source row was validated above");
        if data
            .downloads
            .iter()
            .filter(|row| row.download_id != *id)
            .chain(
                data.lifecycle_quarantines
                    .values()
                    .map(|quarantine| &quarantine.snapshot),
            )
            .any(|row| {
                row.dest_dir == source.dest_dir || row.dest_dir == relocation.request.target_dir
            })
        {
            return Err(relocation_error(
                "Relocation source or target has another legacy owner",
            ));
        }
        for destination in [&relocation.request.source, &relocation.request.target] {
            if !relocation_destinations.insert(destination) {
                return Err(relocation_error("Pending relocations overlap destinations"));
            }
            if data
                .queue_admissions
                .values()
                .any(|entry| &entry.destination == destination)
                || data
                    .admission_attempts
                    .values()
                    .any(|entry| &entry.request.destination == destination)
            {
                return Err(relocation_error(
                    "Relocation destination has a queued admission owner",
                ));
            }
        }
    }
    let mut attempt_ids = HashSet::new();
    for attempt_id in data.admission_attempts.keys().chain(
        data.queue_admissions
            .values()
            .chain(data.released_queue_admissions.values())
            .map(|admission| &admission.attempt_id),
    ) {
        if !attempt_ids.insert(attempt_id) {
            return Err(crate::PumasError::Validation {
                field: "downloads.admission_attempts".into(),
                message: "Admission attempt identity is owned by more than one record".into(),
            });
        }
    }
    if data.schema_version != DOWNLOAD_STORE_SCHEMA_VERSION {
        return Err(crate::PumasError::Validation {
            field: "downloads.schema_version".to_string(),
            message: format!(
                "Unsupported download store schema version {}",
                data.schema_version
            ),
        });
    }
    let mut download_ids = HashSet::new();
    for download in &data.downloads {
        if !download_ids.insert(download.download_id.as_str()) {
            return Err(crate::PumasError::Validation {
                field: "downloads.download_id".to_string(),
                message: format!("Duplicate persisted download ID {}", download.download_id),
            });
        }
        if data
            .recovery_revocations
            .contains_key(&download.download_id)
        {
            return Err(crate::PumasError::Validation {
                field: "downloads.recovery_revocations".to_string(),
                message: format!(
                    "Download {} is both active and recovery-revoked",
                    download.download_id
                ),
            });
        }
    }
    for (download_id, revocation) in &data.recovery_revocations {
        if download_id.trim().is_empty() {
            return Err(crate::PumasError::Validation {
                field: "downloads.recovery_revocations".to_string(),
                message: "Recovery revocation has an empty download ID".to_string(),
            });
        }
        Uuid::parse_str(&revocation.attempt_id).map_err(|source| {
            crate::PumasError::Validation {
                field: "downloads.recovery_revocations".to_string(),
                message: format!("Invalid recovery revocation attempt for {download_id}: {source}"),
            }
        })?;
    }
    let mut queue_positions = BTreeSet::new();
    let mut queue_owners = BTreeMap::new();
    for (attempt_id, attempt) in &data.admission_attempts {
        Uuid::parse_str(attempt_id).map_err(|source| crate::PumasError::Validation {
            field: "downloads.admission_attempts".to_string(),
            message: format!("Invalid admission attempt {attempt_id}: {source}"),
        })?;
        validate_admission_request(&attempt.request)?;
        let download_id = attempt.request.snapshot.download_id.as_str();
        if download_ids.contains(download_id)
            || data.lifecycle_quarantines.contains_key(download_id)
            || data.queue_admissions.contains_key(download_id)
        {
            return Err(crate::PumasError::Validation {
                field: "downloads.admission_attempts".to_string(),
                message: format!("Hidden admission {download_id} has another snapshot owner"),
            });
        }
        let position_key = (
            attempt.request.destination.clone(),
            attempt.position.ordinal,
        );
        if !queue_positions.insert(position_key) {
            return Err(crate::PumasError::Validation {
                field: "downloads.queue_admissions".to_string(),
                message: "Duplicate destination admission ordinal".to_string(),
            });
        }
        queue_owners.insert(
            (download_id.to_string(), attempt_id.clone()),
            (
                attempt.request.destination.clone(),
                attempt.position.clone(),
            ),
        );
    }
    for (download_id, admission) in &data.queue_admissions {
        Uuid::parse_str(&admission.attempt_id).map_err(|source| crate::PumasError::Validation {
            field: "downloads.queue_admissions".to_string(),
            message: format!("Invalid durable admission attempt for {download_id}: {source}"),
        })?;
        let request = DownloadAdmissionRequest {
            snapshot: data
                .downloads
                .iter()
                .find(|snapshot| snapshot.download_id == *download_id)
                .or_else(|| {
                    data.lifecycle_quarantines
                        .get(download_id)
                        .map(|quarantine| &quarantine.snapshot)
                })
                .ok_or_else(|| crate::PumasError::Validation {
                    field: "downloads.queue_admissions".to_string(),
                    message: format!("Durable admission {download_id} has no full-snapshot owner"),
                })?
                .clone(),
            domain: admission.domain,
            destination: admission.destination.clone(),
            requested_payload_files: admission.requested_payload_files.clone(),
            execution_files: admission.execution_files.clone(),
        };
        validate_admission_request(&request)?;
        let position_key = (admission.destination.clone(), admission.position.ordinal);
        if !queue_positions.insert(position_key) {
            return Err(crate::PumasError::Validation {
                field: "downloads.queue_admissions".to_string(),
                message: "Duplicate destination admission ordinal".to_string(),
            });
        }
        queue_owners.insert(
            (download_id.clone(), admission.attempt_id.clone()),
            (admission.destination.clone(), admission.position.clone()),
        );
    }
    for (download_id, admission) in &data.released_queue_admissions {
        Uuid::parse_str(&admission.attempt_id).map_err(|source| crate::PumasError::Validation {
            field: "downloads.released_queue_admissions".into(),
            message: source.to_string(),
        })?;
        if data.queue_admissions.contains_key(download_id)
            || download_ids.contains(download_id.as_str())
            || data
                .admission_attempts
                .values()
                .any(|attempt| attempt.request.snapshot.download_id == *download_id)
        {
            return Err(crate::PumasError::Validation {
                field: "downloads.released_queue_admissions".into(),
                message: "Released admission has an active owner".into(),
            });
        }
        if !queue_positions.insert((admission.destination.clone(), admission.position.ordinal)) {
            return Err(crate::PumasError::Validation {
                field: "downloads.released_queue_admissions".into(),
                message: "Duplicate destination admission ordinal".into(),
            });
        }
        queue_owners.insert(
            (download_id.clone(), admission.attempt_id.clone()),
            (admission.destination.clone(), admission.position.clone()),
        );
    }
    for ((download_id, attempt_id), (destination, position)) in &queue_owners {
        let Some(predecessor) = position.predecessor.as_ref() else {
            continue;
        };
        let Some((predecessor_destination, predecessor_position)) = queue_owners.get(&(
            predecessor.download_id.clone(),
            predecessor.admission_attempt_id.clone(),
        )) else {
            return Err(crate::PumasError::Validation {
                field: "downloads.queue_admissions".to_string(),
                message: format!("Admission {download_id}/{attempt_id} has an orphan predecessor"),
            });
        };
        if predecessor_destination != destination
            || predecessor_position.ordinal >= position.ordinal
        {
            return Err(crate::PumasError::Validation {
                field: "downloads.queue_admissions".to_string(),
                message: format!("Admission {download_id}/{attempt_id} has an invalid predecessor"),
            });
        }
    }
    for (download_id, quarantine) in &data.lifecycle_quarantines {
        if quarantine.snapshot.download_id != *download_id {
            return Err(crate::PumasError::Validation {
                field: "downloads.lifecycle_quarantines".to_string(),
                message: format!(
                    "Lifecycle quarantine key {download_id} does not match its snapshot ID"
                ),
            });
        }
        let expected_status = if quarantine.sticky_failure {
            DownloadStatus::Error
        } else {
            DownloadStatus::Cancelling
        };
        if quarantine.snapshot.status != expected_status {
            return Err(crate::PumasError::Validation {
                field: "downloads.lifecycle_quarantines".to_string(),
                message: format!(
                    "Lifecycle quarantine {download_id} has a status inconsistent with its provenance"
                ),
            });
        }
        if !quarantine.sticky_failure
            && matches!(
                quarantine.disposition,
                PersistedLifecycleCleanupDisposition::VerifiedIntent
                    | PersistedLifecycleCleanupDisposition::Verified
            )
        {
            return Err(crate::PumasError::Validation {
                field: "downloads.lifecycle_quarantines".to_string(),
                message: format!(
                    "Clean lifecycle quarantine {download_id} cannot carry verified failure cleanup"
                ),
            });
        }
        if download_ids.contains(download_id.as_str()) {
            return Err(crate::PumasError::Validation {
                field: "downloads.lifecycle_quarantines".to_string(),
                message: format!(
                    "Lifecycle quarantine {download_id} also has an ordinary resumable row"
                ),
            });
        }
        let revocation = data.recovery_revocations.get(download_id);
        let has_durable_revocation = revocation.is_some_and(|revocation| {
            revocation.disposition == PersistedRevocationDisposition::Durable
        });
        if has_durable_revocation != (quarantine.domain == LifecycleQuarantineDomain::Recovery) {
            return Err(crate::PumasError::Validation {
                field: "downloads.lifecycle_quarantines".to_string(),
                message: format!(
                    "Lifecycle quarantine {download_id} conflicts with its authority domain"
                ),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::io::{BufRead, BufReader, Write};
    use std::process::{Command, Stdio};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn admission_error_conversion_preserves_io_classification_and_uncertainty() {
        for phase in [
            DownloadAdmissionPhase::Intent,
            DownloadAdmissionPhase::Confirmation,
        ] {
            for publication in 0..3 {
                let error = crate::PumasError::Io {
                    message: "publication fault".into(),
                    path: Some(PathBuf::from("downloads.json")),
                    source: Some(std::io::Error::from_raw_os_error(13)),
                };
                let attempt_id = "exact-attempt".to_string();
                let transition = match publication {
                    0 => DownloadAdmissionTransition::NotPublished {
                        attempt_id,
                        phase,
                        stage: AtomicPublishStage::Staging,
                        kind: AtomicPublishFailureKind::Filesystem,
                        error,
                        cleanup: StagingCleanup::Removed,
                    },
                    1 => DownloadAdmissionTransition::PublishedDurabilityUnknown {
                        attempt_id,
                        phase,
                        error,
                    },
                    _ => DownloadAdmissionTransition::VisibilityUnknown {
                        attempt_id,
                        phase,
                        error,
                        cleanup: StagingCleanup::NotRequired,
                    },
                };
                let crate::PumasError::Io {
                    message,
                    path,
                    source,
                } = transition.into_result().unwrap_err()
                else {
                    panic!("publication error lost its IO classification");
                };
                assert_eq!(path, Some(PathBuf::from("downloads.json")));
                assert_eq!(source.unwrap().raw_os_error(), Some(13));
                assert!(message.contains("exact-attempt"));
                assert!(message.contains(&format!("{phase:?}")));
                assert!(message.contains("publication fault"));
                assert!(message.contains(match publication {
                    0 => "not published",
                    1 => "unknown durability",
                    _ => "unknown visibility",
                }));
            }
        }
        let error = DownloadAdmissionTransition::VisibilityUnknown {
            attempt_id: "cleanup-attempt".into(),
            phase: DownloadAdmissionPhase::Confirmation,
            error: crate::PumasError::Other("publication fault".into()),
            cleanup: StagingCleanup::Failed {
                error: crate::PumasError::Other("cleanup fault".into()),
            },
        }
        .into_result()
        .unwrap_err()
        .to_string();
        assert!(error.contains("unknown visibility"));
        assert!(error.contains("publication fault"));
        assert!(error.contains("cleanup fault"));
    }

    fn make_request() -> DownloadRequest {
        DownloadRequest {
            repo_id: "test/model".to_string(),
            family: "test".to_string(),
            official_name: "Test Model".to_string(),
            model_type: Some("llm".to_string()),
            quant: Some("Q4_K_M".to_string()),
            filename: None,
            filenames: None,
            pipeline_tag: None,
            bundle_format: None,
            pipeline_class: None,
            release_date: None,
            download_url: None,
            model_card_json: None,
            license_status: None,
        }
    }

    #[test]
    fn test_save_and_load() {
        let tmp = TempDir::new().unwrap();
        let store = DownloadPersistence::new(tmp.path());

        let entry = PersistedDownload {
            download_id: "dl-1".to_string(),
            repo_id: "test/model".to_string(),
            filename: "model.gguf".to_string(),
            filenames: vec!["model.gguf".to_string()],
            dest_dir: tmp.path().to_path_buf(),
            total_bytes: Some(1000),
            status: DownloadStatus::Paused,
            download_request: make_request(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            known_sha256: None,
            huggingface_evidence: None,
        };

        store.save(&entry).unwrap();
        let loaded = store.load_all();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].download_id, "dl-1");
        assert_eq!(loaded[0].status, DownloadStatus::Paused);
    }

    #[test]
    fn test_upsert() {
        let tmp = TempDir::new().unwrap();
        let store = DownloadPersistence::new(tmp.path());

        let mut entry = PersistedDownload {
            download_id: "dl-1".to_string(),
            repo_id: "test/model".to_string(),
            filename: "model.gguf".to_string(),
            filenames: vec!["model.gguf".to_string()],
            dest_dir: tmp.path().to_path_buf(),
            total_bytes: Some(1000),
            status: DownloadStatus::Paused,
            download_request: make_request(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            known_sha256: None,
            huggingface_evidence: None,
        };

        store.save(&entry).unwrap();
        entry.status = DownloadStatus::Error;
        store.save(&entry).unwrap();

        let loaded = store.load_all();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].status, DownloadStatus::Error);
    }

    #[test]
    fn test_remove() {
        let tmp = TempDir::new().unwrap();
        let store = DownloadPersistence::new(tmp.path());

        let entry = PersistedDownload {
            download_id: "dl-1".to_string(),
            repo_id: "test/model".to_string(),
            filename: "model.gguf".to_string(),
            filenames: vec!["model.gguf".to_string()],
            dest_dir: tmp.path().to_path_buf(),
            total_bytes: Some(1000),
            status: DownloadStatus::Paused,
            download_request: make_request(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            known_sha256: None,
            huggingface_evidence: None,
        };

        store.save(&entry).unwrap();
        assert_eq!(store.load_all().len(), 1);

        store.remove("dl-1").unwrap();
        assert_eq!(store.load_all().len(), 0);
    }

    #[test]
    fn test_load_empty() {
        let tmp = TempDir::new().unwrap();
        let store = DownloadPersistence::new(tmp.path());
        assert_eq!(store.load_all().len(), 0);
    }

    #[test]
    fn revoked_download_rejects_stale_save_status_and_relocation() {
        let tmp = TempDir::new().unwrap();
        let store = DownloadPersistence::new(tmp.path());
        let stale = PersistedDownload {
            download_id: "dl-revoked".to_string(),
            repo_id: "test/model".to_string(),
            filename: "model.gguf".to_string(),
            filenames: vec!["model.gguf".to_string()],
            dest_dir: tmp.path().join("model"),
            total_bytes: Some(1000),
            status: DownloadStatus::Paused,
            download_request: make_request(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            known_sha256: None,
            huggingface_evidence: None,
        };
        store.save(&stale).unwrap();

        store.revoke("dl-revoked").unwrap();

        assert!(store.save(&stale).is_err());
        assert!(!store
            .update_status("dl-revoked", DownloadStatus::Error)
            .unwrap());
        assert!(store
            .begin_legacy_relocation(
                &Uuid::new_v4().to_string(),
                &stale,
                &legacy_relocation_request(tmp.path().join("elsewhere")),
            )
            .is_err());
        assert!(store.load_all().is_empty());
    }

    #[test]
    fn strict_revoke_propagates_corrupt_store_without_recording_revocation() {
        let tmp = TempDir::new().unwrap();
        let store = DownloadPersistence::new(tmp.path());
        std::fs::write(&store.path, b"{not-json").unwrap();

        assert!(store.revoke("dl-corrupt").is_err());

        std::fs::remove_file(&store.path).unwrap();
        let entry = PersistedDownload {
            download_id: "dl-corrupt".to_string(),
            repo_id: "test/model".to_string(),
            filename: "model.gguf".to_string(),
            filenames: vec!["model.gguf".to_string()],
            dest_dir: tmp.path().join("model"),
            total_bytes: Some(1000),
            status: DownloadStatus::Paused,
            download_request: make_request(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            known_sha256: None,
            huggingface_evidence: None,
        };
        store.save(&entry).unwrap();
        assert_eq!(store.load_all().len(), 1);
    }

    fn persisted(download_id: &str) -> PersistedDownload {
        PersistedDownload {
            download_id: download_id.to_string(),
            repo_id: "test/model".to_string(),
            filename: "model.gguf".to_string(),
            filenames: vec!["model.gguf".to_string()],
            dest_dir: PathBuf::from("/managed/model"),
            total_bytes: Some(1000),
            status: DownloadStatus::Paused,
            download_request: make_request(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            known_sha256: None,
            huggingface_evidence: None,
        }
    }

    #[derive(Debug, Clone, Copy)]
    enum ScriptedPublication {
        Durable,
        NotPublished,
        PublishedDurabilityUnknown,
        VisibilityUnknownBeforeEffect,
        VisibilityUnknownAfterEffect,
    }

    struct ScriptedPublisher {
        script: Mutex<VecDeque<ScriptedPublication>>,
        calls: AtomicUsize,
    }

    impl ScriptedPublisher {
        fn new(script: impl IntoIterator<Item = ScriptedPublication>) -> Self {
            Self {
                script: Mutex::new(script.into_iter().collect()),
                calls: AtomicUsize::new(0),
            }
        }

        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    impl DownloadStorePublisher for ScriptedPublisher {
        fn publish(
            &self,
            target: &AtomicJsonTarget,
            data: &DownloadStoreData,
        ) -> AtomicPublishResult {
            self.calls.fetch_add(1, Ordering::SeqCst);
            match self
                .script
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or(ScriptedPublication::Durable)
            {
                ScriptedPublication::Durable => target.publish_json(data),
                ScriptedPublication::NotPublished => Err(Box::new(AtomicPublishFailure {
                    stage: AtomicPublishStage::Staging,
                    kind: AtomicPublishFailureKind::Filesystem,
                    error: crate::PumasError::Other("injected pre-publication failure".to_string()),
                    cleanup: StagingCleanup::NotRequired,
                })),
                ScriptedPublication::PublishedDurabilityUnknown => {
                    assert!(matches!(
                        target.publish_json(data).unwrap(),
                        AtomicPublication::Durable
                    ));
                    Ok(AtomicPublication::PublishedDurabilityUnknown {
                        error: crate::PumasError::Other(
                            "injected parent-sync uncertainty".to_string(),
                        ),
                    })
                }
                ScriptedPublication::VisibilityUnknownBeforeEffect => {
                    Ok(AtomicPublication::VisibilityUnknown {
                        error: crate::PumasError::Other(
                            "injected rename visibility uncertainty".to_string(),
                        ),
                        cleanup: StagingCleanup::NotRequired,
                    })
                }
                ScriptedPublication::VisibilityUnknownAfterEffect => {
                    assert!(matches!(
                        target.publish_json(data).unwrap(),
                        AtomicPublication::Durable
                    ));
                    Ok(AtomicPublication::VisibilityUnknown {
                        error: crate::PumasError::Other(
                            "injected post-effect rename visibility uncertainty".to_string(),
                        ),
                        cleanup: StagingCleanup::NotRequired,
                    })
                }
            }
        }
    }

    #[test]
    fn absent_row_requires_unknown_then_durable_publications() {
        let tmp = TempDir::new().unwrap();
        let publisher = Arc::new(ScriptedPublisher::new([
            ScriptedPublication::Durable,
            ScriptedPublication::Durable,
        ]));
        let store = DownloadPersistence::new(tmp.path()).with_test_publisher(publisher.clone());

        let outcome = store.revoke_for_recovery("dl-absent").unwrap();

        assert!(matches!(
            outcome,
            RecoveryRevocation::Durable {
                source: RecoveryRevocationSource::NewlyPublished,
                ..
            }
        ));
        assert_eq!(publisher.calls(), 2);
        assert!(matches!(
            DownloadPersistence::new(tmp.path())
                .revoke_for_recovery("dl-absent")
                .unwrap(),
            RecoveryRevocation::Durable {
                source: RecoveryRevocationSource::Persisted,
                ..
            }
        ));
    }

    #[test]
    fn unknown_revocation_survives_a_fresh_owner_and_must_be_republished() {
        let tmp = TempDir::new().unwrap();
        let store = DownloadPersistence::new(tmp.path());
        let entry = persisted("dl-unknown");
        store.save(&entry).unwrap();
        let publisher = Arc::new(ScriptedPublisher::new([
            ScriptedPublication::PublishedDurabilityUnknown,
        ]));
        let uncertain = store.with_test_publisher(publisher);

        let outcome = uncertain.revoke_for_recovery("dl-unknown").unwrap();

        assert!(matches!(
            outcome,
            RecoveryRevocation::PublishedDurabilityUnknown {
                phase: RecoveryRevocationPhase::Intent,
                ..
            }
        ));
        let fresh = DownloadPersistence::new(tmp.path());
        assert!(fresh.is_revoked("dl-unknown").unwrap());
        assert!(fresh.save(&entry).is_err());
        assert!(matches!(
            fresh.revoke_for_recovery("dl-unknown").unwrap(),
            RecoveryRevocation::Durable {
                source: RecoveryRevocationSource::NewlyPublished,
                ..
            }
        ));
    }

    #[test]
    fn failed_confirmation_leaves_unknown_instead_of_promoting_absence() {
        let tmp = TempDir::new().unwrap();
        let publisher = Arc::new(ScriptedPublisher::new([
            ScriptedPublication::Durable,
            ScriptedPublication::NotPublished,
        ]));
        let store = DownloadPersistence::new(tmp.path()).with_test_publisher(publisher);

        let outcome = store.revoke_for_recovery("dl-confirmation").unwrap();

        assert!(matches!(
            outcome,
            RecoveryRevocation::NotPublished {
                phase: RecoveryRevocationPhase::Confirmation,
                ..
            }
        ));
        let fresh = DownloadPersistence::new(tmp.path());
        assert!(fresh.is_revoked("dl-confirmation").unwrap());
        assert!(matches!(
            fresh.revoke_for_recovery("dl-confirmation").unwrap(),
            RecoveryRevocation::Durable {
                source: RecoveryRevocationSource::NewlyPublished,
                ..
            }
        ));
    }

    fn confirmation_ambiguity_outcomes(
        download_id: &str,
        confirmation: ScriptedPublication,
    ) -> (RecoveryRevocation, RecoveryRevocation) {
        let tmp = TempDir::new().unwrap();
        let entry = persisted(download_id);
        DownloadPersistence::new(tmp.path()).save(&entry).unwrap();
        let publisher = Arc::new(ScriptedPublisher::new([
            ScriptedPublication::Durable,
            confirmation,
        ]));
        let store = DownloadPersistence::new(tmp.path()).with_test_publisher(publisher);

        let initiating = store.revoke_for_recovery(download_id).unwrap();
        let fresh = DownloadPersistence::new(tmp.path());
        assert!(fresh.is_revoked(download_id).unwrap());
        assert!(fresh.save(&entry).is_err());
        let retried = fresh.revoke_for_recovery(download_id).unwrap();
        (initiating, retried)
    }

    #[test]
    fn confirmation_parent_sync_unknown_never_succeeds_the_initiating_call() {
        let (initiating, retried) = confirmation_ambiguity_outcomes(
            "dl-confirmation-parent-sync",
            ScriptedPublication::PublishedDurabilityUnknown,
        );

        assert!(matches!(
            initiating,
            RecoveryRevocation::PublishedDurabilityUnknown {
                phase: RecoveryRevocationPhase::Confirmation,
                ..
            }
        ));
        assert!(matches!(
            retried,
            RecoveryRevocation::Durable {
                source: RecoveryRevocationSource::Persisted,
                ..
            }
        ));
    }

    #[test]
    fn confirmation_pre_effect_visibility_unknown_retries_from_durable_intent() {
        let (initiating, retried) = confirmation_ambiguity_outcomes(
            "dl-confirmation-before-effect",
            ScriptedPublication::VisibilityUnknownBeforeEffect,
        );

        assert!(matches!(
            initiating,
            RecoveryRevocation::VisibilityUnknown {
                phase: RecoveryRevocationPhase::Confirmation,
                ..
            }
        ));
        assert!(matches!(
            retried,
            RecoveryRevocation::Durable {
                source: RecoveryRevocationSource::NewlyPublished,
                ..
            }
        ));
    }

    #[test]
    fn confirmation_post_effect_visibility_unknown_never_succeeds_the_initiating_call() {
        let (initiating, retried) = confirmation_ambiguity_outcomes(
            "dl-confirmation-after-effect",
            ScriptedPublication::VisibilityUnknownAfterEffect,
        );

        assert!(matches!(
            initiating,
            RecoveryRevocation::VisibilityUnknown {
                phase: RecoveryRevocationPhase::Confirmation,
                ..
            }
        ));
        assert!(matches!(
            retried,
            RecoveryRevocation::Durable {
                source: RecoveryRevocationSource::Persisted,
                ..
            }
        ));
    }

    #[test]
    fn prepublication_and_visibility_unknown_outcomes_never_admit_recovery() {
        for scripted in [
            ScriptedPublication::NotPublished,
            ScriptedPublication::VisibilityUnknownBeforeEffect,
        ] {
            let tmp = TempDir::new().unwrap();
            let entry = persisted("dl-unpublished");
            DownloadPersistence::new(tmp.path()).save(&entry).unwrap();
            let store = DownloadPersistence::new(tmp.path())
                .with_test_publisher(Arc::new(ScriptedPublisher::new([scripted])));

            let outcome = store.revoke_for_recovery("dl-unpublished").unwrap();

            assert!(matches!(
                outcome,
                RecoveryRevocation::NotPublished {
                    phase: RecoveryRevocationPhase::Intent,
                    ..
                } | RecoveryRevocation::VisibilityUnknown {
                    phase: RecoveryRevocationPhase::Intent,
                    ..
                }
            ));
            assert_eq!(
                DownloadPersistence::new(tmp.path())
                    .load_all_strict()
                    .unwrap()
                    .len(),
                1
            );
        }
    }

    #[test]
    fn revocation_preserves_the_closed_prepublication_failure_classification() {
        let tmp = TempDir::new().unwrap();
        let store = DownloadPersistence::new(tmp.path()).with_test_publisher(Arc::new(
            ScriptedPublisher::new([ScriptedPublication::NotPublished]),
        ));

        let outcome = store.revoke_for_recovery("dl-classified").unwrap();

        assert!(matches!(
            outcome,
            RecoveryRevocation::NotPublished {
                phase: RecoveryRevocationPhase::Intent,
                stage: AtomicPublishStage::Staging,
                kind: AtomicPublishFailureKind::Filesystem,
                ..
            }
        ));
    }

    #[test]
    fn post_effect_visibility_unknown_persists_fail_closed_for_a_fresh_owner() {
        let tmp = TempDir::new().unwrap();
        let entry = persisted("dl-visible-unknown");
        DownloadPersistence::new(tmp.path()).save(&entry).unwrap();
        let store = DownloadPersistence::new(tmp.path()).with_test_publisher(Arc::new(
            ScriptedPublisher::new([ScriptedPublication::VisibilityUnknownAfterEffect]),
        ));

        let outcome = store.revoke_for_recovery("dl-visible-unknown").unwrap();

        assert!(matches!(
            outcome,
            RecoveryRevocation::VisibilityUnknown {
                phase: RecoveryRevocationPhase::Intent,
                ..
            }
        ));
        let fresh = DownloadPersistence::new(tmp.path());
        assert!(fresh.is_revoked("dl-visible-unknown").unwrap());
        assert!(fresh.save(&entry).is_err());
    }

    struct BlockingObserver {
        operation: StoreOperation,
        entered: Mutex<Option<mpsc::Sender<()>>>,
        release: Mutex<Option<mpsc::Receiver<()>>>,
    }

    impl StoreTransactionObserver for BlockingObserver {
        fn acquired(&self, operation: StoreOperation) {
            if operation != self.operation {
                return;
            }
            if let Some(entered) = self.entered.lock().unwrap().take() {
                entered.send(()).unwrap();
            }
            if let Some(release) = self.release.lock().unwrap().take() {
                release.recv().unwrap();
            }
        }
    }

    struct LockLifecycleObserver {
        attempting: mpsc::Sender<()>,
        acquired: mpsc::Sender<()>,
    }

    impl StoreTransactionObserver for LockLifecycleObserver {
        fn attempting(&self, _operation: StoreOperation) {
            self.attempting.send(()).unwrap();
        }

        fn acquired(&self, _operation: StoreOperation) {
            self.acquired.send(()).unwrap();
        }
    }

    #[test]
    fn independent_writer_queued_after_actual_revoke_lock_cannot_recreate() {
        let tmp = TempDir::new().unwrap();
        let entry = persisted("dl-cross-owner");
        DownloadPersistence::new(tmp.path()).save(&entry).unwrap();
        let (revoke_entered_tx, revoke_entered_rx) = mpsc::channel();
        let (release_revoke_tx, release_revoke_rx) = mpsc::channel();
        let revoke_store =
            DownloadPersistence::new(tmp.path()).with_test_observer(Arc::new(BlockingObserver {
                operation: StoreOperation::Revoke,
                entered: Mutex::new(Some(revoke_entered_tx)),
                release: Mutex::new(Some(release_revoke_rx)),
            }));
        let revoke = thread::spawn(move || revoke_store.revoke_for_recovery("dl-cross-owner"));
        revoke_entered_rx.recv().unwrap();

        let (writer_attempting_tx, writer_attempting_rx) = mpsc::channel();
        let (writer_acquired_tx, writer_acquired_rx) = mpsc::channel();
        let writer_store = DownloadPersistence::new(tmp.path()).with_test_observer(Arc::new(
            LockLifecycleObserver {
                attempting: writer_attempting_tx,
                acquired: writer_acquired_tx,
            },
        ));
        let writer = thread::spawn(move || writer_store.save(&entry));
        writer_attempting_rx
            .recv_timeout(Duration::from_secs(5))
            .unwrap();
        assert!(writer_acquired_rx.try_recv().is_err());

        release_revoke_tx.send(()).unwrap();
        assert!(matches!(
            revoke.join().unwrap().unwrap(),
            RecoveryRevocation::Durable { .. }
        ));
        writer_acquired_rx
            .recv_timeout(Duration::from_secs(5))
            .unwrap();
        assert!(writer.join().unwrap().is_err());
        assert!(DownloadPersistence::new(tmp.path())
            .load_all_strict()
            .unwrap()
            .is_empty());
    }

    #[test]
    fn revoke_queued_after_actual_writer_lock_removes_the_committed_write() {
        let tmp = TempDir::new().unwrap();
        let mut entry = persisted("dl-writer-first");
        DownloadPersistence::new(tmp.path()).save(&entry).unwrap();
        entry.status = DownloadStatus::Error;
        let (writer_entered_tx, writer_entered_rx) = mpsc::channel();
        let (release_writer_tx, release_writer_rx) = mpsc::channel();
        let writer_store =
            DownloadPersistence::new(tmp.path()).with_test_observer(Arc::new(BlockingObserver {
                operation: StoreOperation::Save,
                entered: Mutex::new(Some(writer_entered_tx)),
                release: Mutex::new(Some(release_writer_rx)),
            }));
        let writer = thread::spawn(move || writer_store.save(&entry));
        writer_entered_rx.recv().unwrap();

        let (revoke_attempting_tx, revoke_attempting_rx) = mpsc::channel();
        let (revoke_acquired_tx, revoke_acquired_rx) = mpsc::channel();
        let revoke_store = DownloadPersistence::new(tmp.path()).with_test_observer(Arc::new(
            LockLifecycleObserver {
                attempting: revoke_attempting_tx,
                acquired: revoke_acquired_tx,
            },
        ));
        let revoke = thread::spawn(move || revoke_store.revoke_for_recovery("dl-writer-first"));
        revoke_attempting_rx
            .recv_timeout(Duration::from_secs(5))
            .unwrap();
        assert!(revoke_acquired_rx.try_recv().is_err());

        release_writer_tx.send(()).unwrap();
        writer.join().unwrap().unwrap();
        revoke_acquired_rx
            .recv_timeout(Duration::from_secs(5))
            .unwrap();
        assert!(matches!(
            revoke.join().unwrap().unwrap(),
            RecoveryRevocation::Durable { .. }
        ));
        assert!(DownloadPersistence::new(tmp.path())
            .load_all_strict()
            .unwrap()
            .is_empty());
    }

    struct ChildLockObserver;

    impl StoreTransactionObserver for ChildLockObserver {
        fn acquired(&self, operation: StoreOperation) {
            if operation != StoreOperation::Save {
                return;
            }
            println!("PUMAS_STORE_LOCK_ACQUIRED");
            std::io::stdout().flush().unwrap();
            let mut release = String::new();
            std::io::stdin().read_line(&mut release).unwrap();
        }
    }

    #[test]
    #[ignore = "subprocess helper invoked by os_lock_is_released_when_writer_process_dies"]
    fn download_store_child_lock_holder() {
        let Some(data_dir) = std::env::var_os("PUMAS_STORE_CHILD_DIR") else {
            return;
        };
        let store = DownloadPersistence::new(Path::new(&data_dir))
            .with_test_observer(Arc::new(ChildLockObserver));
        store.save(&persisted("dl-child-lock")).unwrap();
    }

    #[test]
    fn os_lock_is_released_when_writer_process_dies() {
        let tmp = TempDir::new().unwrap();
        DownloadPersistence::new(tmp.path())
            .save(&persisted("dl-child-lock"))
            .unwrap();
        let mut child = Command::new(std::env::current_exe().unwrap())
            .arg("--ignored")
            .arg("--exact")
            .arg("model_library::download_store::tests::download_store_child_lock_holder")
            .arg("--nocapture")
            .env("PUMAS_STORE_CHILD_DIR", tmp.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let mut output = BufReader::new(child.stdout.take().unwrap());
        let mut line = String::new();
        loop {
            line.clear();
            assert_ne!(output.read_line(&mut line).unwrap(), 0);
            if line.contains("PUMAS_STORE_LOCK_ACQUIRED") {
                break;
            }
        }

        let (parent_attempting_tx, parent_attempting_rx) = mpsc::channel();
        let (parent_acquired_tx, parent_acquired_rx) = mpsc::channel();
        let parent_store = DownloadPersistence::new(tmp.path()).with_test_observer(Arc::new(
            LockLifecycleObserver {
                attempting: parent_attempting_tx,
                acquired: parent_acquired_tx,
            },
        ));
        let revoke = thread::spawn(move || parent_store.revoke_for_recovery("dl-child-lock"));
        parent_attempting_rx
            .recv_timeout(Duration::from_secs(5))
            .unwrap();
        assert!(parent_acquired_rx.try_recv().is_err());

        child.kill().unwrap();
        child.wait().unwrap();
        parent_acquired_rx
            .recv_timeout(Duration::from_secs(5))
            .unwrap();
        assert!(matches!(
            revoke.join().unwrap().unwrap(),
            RecoveryRevocation::Durable { .. }
        ));
        assert!(DownloadPersistence::new(tmp.path())
            .load_all_strict()
            .unwrap()
            .is_empty());
    }

    struct ExitAfterPublisher {
        exit_after: usize,
        calls: AtomicUsize,
    }

    impl DownloadStorePublisher for ExitAfterPublisher {
        fn publish(
            &self,
            target: &AtomicJsonTarget,
            data: &DownloadStoreData,
        ) -> AtomicPublishResult {
            let outcome = target.publish_json(data);
            let call = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
            if call == self.exit_after {
                std::process::exit(80 + i32::try_from(call).expect("test call count fits i32"));
            }
            outcome
        }
    }

    #[test]
    #[ignore = "subprocess helper invoked by interruption_preserves_last_durable_revocation_phase"]
    fn download_store_interruption_child() {
        let Some(data_dir) = std::env::var_os("PUMAS_STORE_CHILD_DIR") else {
            return;
        };
        let exit_after = std::env::var("PUMAS_STORE_EXIT_AFTER")
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let store = DownloadPersistence::new(Path::new(&data_dir)).with_test_publisher(Arc::new(
            ExitAfterPublisher {
                exit_after,
                calls: AtomicUsize::new(0),
            },
        ));
        let _ = store.revoke_for_recovery("dl-interrupted");
    }

    fn persisted_disposition(
        store: &DownloadPersistence,
        download_id: &str,
    ) -> Option<PersistedRevocationDisposition> {
        let transaction = store.transaction(StoreOperation::Load).unwrap();
        store
            .load_data_strict(&transaction)
            .unwrap()
            .recovery_revocations
            .get(download_id)
            .map(|revocation| revocation.disposition)
    }

    #[test]
    fn interruption_preserves_last_durable_revocation_phase() {
        for (exit_after, expected) in [
            (1_usize, PersistedRevocationDisposition::DurabilityUnknown),
            (2_usize, PersistedRevocationDisposition::Durable),
        ] {
            let tmp = TempDir::new().unwrap();
            let status = Command::new(std::env::current_exe().unwrap())
                .arg("--ignored")
                .arg("--exact")
                .arg("model_library::download_store::tests::download_store_interruption_child")
                .arg("--nocapture")
                .env("PUMAS_STORE_CHILD_DIR", tmp.path())
                .env("PUMAS_STORE_EXIT_AFTER", exit_after.to_string())
                .status()
                .unwrap();
            assert_eq!(status.code(), Some(80 + i32::try_from(exit_after).unwrap()));

            let fresh = DownloadPersistence::new(tmp.path());
            assert_eq!(
                persisted_disposition(&fresh, "dl-interrupted"),
                Some(expected)
            );
            if expected == PersistedRevocationDisposition::DurabilityUnknown {
                assert!(matches!(
                    fresh.revoke_for_recovery("dl-interrupted").unwrap(),
                    RecoveryRevocation::Durable {
                        source: RecoveryRevocationSource::NewlyPublished,
                        ..
                    }
                ));
            } else {
                assert!(matches!(
                    fresh.revoke_for_recovery("dl-interrupted").unwrap(),
                    RecoveryRevocation::Durable {
                        source: RecoveryRevocationSource::Persisted,
                        ..
                    }
                ));
            }
        }
    }

    #[test]
    fn legacy_store_migrates_strictly_on_the_next_mutation() {
        let tmp = TempDir::new().unwrap();
        let legacy = serde_json::json!({"downloads": [persisted("dl-legacy")]});
        std::fs::write(
            tmp.path().join("downloads.json"),
            serde_json::to_vec_pretty(&legacy).unwrap(),
        )
        .unwrap();
        let store = DownloadPersistence::new(tmp.path());

        assert_eq!(store.load_all_strict().unwrap().len(), 1);
        assert!(store
            .update_status("dl-legacy", DownloadStatus::Error)
            .unwrap());

        let value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(tmp.path().join("downloads.json")).unwrap())
                .unwrap();
        assert_eq!(value["schema_version"], DOWNLOAD_STORE_SCHEMA_VERSION);
        assert!(value.get("store_generation").is_none());
        assert!(value["recovery_revocations"]
            .as_object()
            .unwrap()
            .is_empty());
    }

    #[test]
    fn strict_fresh_read_rejects_unsupported_and_conflicting_v3_documents() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("downloads.json");
        std::fs::write(
            &path,
            serde_json::to_vec(&serde_json::json!({
                "schema_version": 99,
                "downloads": [],
                "recovery_revocations": {}
            }))
            .unwrap(),
        )
        .unwrap();
        let unsupported = DownloadPersistence::new(tmp.path())
            .load_all_strict()
            .unwrap_err();
        assert!(matches!(
            unsupported,
            crate::PumasError::Validation { ref field, .. }
                if field == "downloads.schema_version"
        ));

        let entry = persisted("dl-conflict");
        std::fs::write(
            &path,
            serde_json::to_vec(&serde_json::json!({
                "schema_version": DOWNLOAD_STORE_SCHEMA_VERSION,
                "downloads": [entry],
                "lifecycle_quarantines": {},
                "admission_attempts": {},
                "queue_admissions": {},
                "recovery_revocations": {
                    "dl-conflict": {
                        "attempt_id": Uuid::new_v4().to_string(),
                        "disposition": "durable"
                    }
                }
            }))
            .unwrap(),
        )
        .unwrap();
        let conflict = DownloadPersistence::new(tmp.path())
            .load_all_strict()
            .unwrap_err();
        assert!(matches!(
            conflict,
            crate::PumasError::Validation { ref field, .. }
                if field == "downloads.recovery_revocations"
        ));
    }

    #[test]
    fn durable_revoke_persists_a_confirmed_tombstone_for_a_fresh_owner() {
        let tmp = TempDir::new().unwrap();
        let store = DownloadPersistence::new(tmp.path());
        store.save(&persisted("dl-durable")).unwrap();

        let outcome = store.revoke_for_recovery("dl-durable").unwrap();

        assert!(matches!(
            outcome,
            RecoveryRevocation::Durable {
                source: RecoveryRevocationSource::NewlyPublished,
                ..
            }
        ));
        let fresh = DownloadPersistence::new(tmp.path());
        assert!(fresh.load_all_strict().unwrap().is_empty());
        assert!(matches!(
            fresh.revoke_for_recovery("dl-durable").unwrap(),
            RecoveryRevocation::Durable {
                source: RecoveryRevocationSource::Persisted,
                ..
            }
        ));
    }

    #[test]
    fn versioned_v1_and_v2_documents_migrate_as_recoverable_v3_state() {
        for version in [1_u32, 2_u32] {
            let tmp = TempDir::new().unwrap();
            let path = tmp.path().join("downloads.json");
            let mut document = serde_json::json!({
                "schema_version": version,
                "downloads": [persisted(&format!("dl-v{version}"))],
            });
            if version == 2 {
                document["recovery_revocations"] = serde_json::json!({});
            }
            std::fs::write(&path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();

            let store = DownloadPersistence::new(tmp.path());
            let inventory = store.load_lifecycle_inventory_strict().unwrap();
            assert_eq!(inventory.downloads.len(), 1);
            assert!(inventory.quarantines.is_empty());

            assert!(store
                .update_status(&format!("dl-v{version}"), DownloadStatus::Error)
                .unwrap());
            let migrated: serde_json::Value =
                serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
            assert_eq!(migrated["schema_version"], 3);
            assert!(migrated["lifecycle_quarantines"]
                .as_object()
                .unwrap()
                .is_empty());
        }
    }

    #[test]
    fn ambient_quarantine_exclusively_owns_the_snapshot_and_rejects_stale_writers() {
        let tmp = TempDir::new().unwrap();
        let store = DownloadPersistence::new(tmp.path());
        let entry = persisted("dl-ambient-quarantine");
        store.save(&entry).unwrap();

        store
            .begin_lifecycle_quarantine(&entry, LifecycleQuarantineDomain::Ambient, true)
            .unwrap();
        let inventory = store.load_lifecycle_inventory_strict().unwrap();
        assert!(inventory.downloads.is_empty());
        assert_eq!(
            inventory.quarantines["dl-ambient-quarantine"].disposition,
            LifecycleCleanupDisposition::Pending
        );
        assert_eq!(
            inventory.quarantines["dl-ambient-quarantine"]
                .snapshot
                .download_id,
            entry.download_id
        );
        assert!(inventory.quarantines["dl-ambient-quarantine"].sticky_failure);
        assert!(store.save(&entry).is_err());
        assert!(!store
            .update_status("dl-ambient-quarantine", DownloadStatus::Paused)
            .unwrap());
        assert!(store
            .begin_legacy_relocation(
                &Uuid::new_v4().to_string(),
                &entry,
                &legacy_relocation_request(tmp.path().join("moved")),
            )
            .is_err());

        assert!(store
            .verify_lifecycle_quarantine("dl-ambient-quarantine")
            .unwrap());
        let fresh_store = DownloadPersistence::new(tmp.path());
        fresh_store.reconcile_lifecycle_inventory_strict().unwrap();
        let fresh = fresh_store.load_lifecycle_inventory_strict().unwrap();
        assert_eq!(
            fresh.quarantines["dl-ambient-quarantine"].disposition,
            LifecycleCleanupDisposition::Verified
        );
    }

    #[test]
    fn recovery_quarantine_preserves_the_durable_revocation_tombstone() {
        let tmp = TempDir::new().unwrap();
        let store = DownloadPersistence::new(tmp.path());
        let entry = persisted("dl-recovery-quarantine");
        store.save(&entry).unwrap();
        store.revoke("dl-recovery-quarantine").unwrap();

        store
            .begin_lifecycle_quarantine(&entry, LifecycleQuarantineDomain::Recovery, true)
            .unwrap();
        assert!(store.is_revoked("dl-recovery-quarantine").unwrap());
        assert!(store
            .verify_lifecycle_quarantine("dl-recovery-quarantine")
            .unwrap());

        let fresh_store = DownloadPersistence::new(tmp.path());
        fresh_store.reconcile_lifecycle_inventory_strict().unwrap();
        let inventory = fresh_store.load_lifecycle_inventory_strict().unwrap();
        let quarantine = &inventory.quarantines["dl-recovery-quarantine"];
        assert_eq!(quarantine.domain, LifecycleQuarantineDomain::Recovery);
        assert!(quarantine.sticky_failure);
        assert_eq!(
            quarantine.disposition,
            LifecycleCleanupDisposition::Verified
        );
        assert!(fresh_store.is_revoked("dl-recovery-quarantine").unwrap());
    }

    #[test]
    fn quarantine_publication_ambiguity_never_authorizes_the_initiating_owner() {
        let tmp = TempDir::new().unwrap();
        let entry = persisted("dl-quarantine-unknown");
        DownloadPersistence::new(tmp.path()).save(&entry).unwrap();
        let pending_publisher = Arc::new(ScriptedPublisher::new([
            ScriptedPublication::Durable,
            ScriptedPublication::PublishedDurabilityUnknown,
        ]));
        let pending_store =
            DownloadPersistence::new(tmp.path()).with_test_publisher(pending_publisher.clone());

        assert!(pending_store
            .begin_lifecycle_quarantine(&entry, LifecycleQuarantineDomain::Ambient, true)
            .is_err());
        assert_eq!(pending_publisher.calls(), 2);
        let pending = DownloadPersistence::new(tmp.path())
            .load_lifecycle_inventory_strict()
            .unwrap();
        assert_eq!(
            pending.quarantines["dl-quarantine-unknown"].disposition,
            LifecycleCleanupDisposition::Pending
        );

        let verified_publisher = Arc::new(ScriptedPublisher::new([
            ScriptedPublication::Durable,
            ScriptedPublication::PublishedDurabilityUnknown,
        ]));
        let verified_store =
            DownloadPersistence::new(tmp.path()).with_test_publisher(verified_publisher.clone());
        assert!(verified_store
            .verify_lifecycle_quarantine("dl-quarantine-unknown")
            .is_err());
        assert_eq!(verified_publisher.calls(), 2);
    }

    #[test]
    fn failed_pending_intent_leaves_the_ordinary_row_owned_by_a_fresh_writer() {
        let tmp = TempDir::new().unwrap();
        let entry = persisted("dl-quarantine-not-published");
        DownloadPersistence::new(tmp.path()).save(&entry).unwrap();
        let publisher = Arc::new(ScriptedPublisher::new([ScriptedPublication::NotPublished]));
        let store = DownloadPersistence::new(tmp.path()).with_test_publisher(publisher.clone());

        assert!(store
            .begin_lifecycle_quarantine(&entry, LifecycleQuarantineDomain::Ambient, false)
            .is_err());
        assert_eq!(publisher.calls(), 1);
        let fresh = DownloadPersistence::new(tmp.path())
            .load_lifecycle_inventory_strict()
            .unwrap();
        assert_eq!(fresh.downloads.len(), 1);
        assert!(fresh.quarantines.is_empty());
    }

    #[test]
    fn clean_pending_quarantine_is_removed_durably_before_cancelled_publication() {
        let tmp = TempDir::new().unwrap();
        let store = DownloadPersistence::new(tmp.path());
        let entry = persisted("dl-clean-cancel");
        store.save(&entry).unwrap();
        store
            .begin_lifecycle_quarantine(&entry, LifecycleQuarantineDomain::Ambient, false)
            .unwrap();
        let pending = store.load_lifecycle_inventory_strict().unwrap();
        assert!(!pending.quarantines["dl-clean-cancel"].sticky_failure);

        assert!(store
            .remove_clean_lifecycle_quarantine("dl-clean-cancel")
            .unwrap());
        let fresh = DownloadPersistence::new(tmp.path())
            .load_lifecycle_inventory_strict()
            .unwrap();
        assert!(fresh.downloads.is_empty());
        assert!(fresh.quarantines.is_empty());
    }

    #[test]
    fn pending_cleanup_can_be_promoted_to_sticky_failure_but_never_downgraded() {
        let tmp = TempDir::new().unwrap();
        let store = DownloadPersistence::new(tmp.path());
        let entry = persisted("dl-late-failure");
        store.save(&entry).unwrap();
        store
            .begin_lifecycle_quarantine(&entry, LifecycleQuarantineDomain::Ambient, false)
            .unwrap();

        assert!(store
            .mark_lifecycle_quarantine_failed("dl-late-failure")
            .unwrap());
        let failed = store.load_lifecycle_inventory_strict().unwrap();
        assert!(failed.quarantines["dl-late-failure"].sticky_failure);
        assert!(store
            .remove_clean_lifecycle_quarantine("dl-late-failure")
            .is_err());
        assert!(store
            .verify_lifecycle_quarantine("dl-late-failure")
            .unwrap());
    }

    fn admission_request(download_id: &str) -> DownloadAdmissionRequest {
        DownloadAdmissionRequest {
            snapshot: persisted(download_id),
            domain: DownloadAdmissionDomain::Ambient,
            destination: PersistedDestinationIdentity {
                library_root: "root-device-7-inode-11".to_string(),
                relative_target: "llm/test/model".to_string(),
            },
            requested_payload_files: vec!["model.gguf".to_string()],
            execution_files: vec!["README.md".to_string(), "model.gguf".to_string()],
        }
    }

    fn legacy_relocation_request(target_dir: PathBuf) -> LegacyRelocationRequest {
        LegacyRelocationRequest {
            source: admission_request("unused").destination,
            target: PersistedDestinationIdentity {
                library_root: "root-device-7-inode-11".into(),
                relative_target: "llm/new/model".into(),
            },
            target_dir,
            model_type: Some("llm".into()),
            family: Some("new".into()),
        }
    }

    #[test]
    fn legacy_relocation_refuses_existing_foreign_source_or_target_owners() {
        for at_target in [false, true] {
            for quarantined in [false, true] {
                let tmp = TempDir::new().unwrap();
                let store = DownloadPersistence::new(tmp.path());
                let source = persisted("source");
                store.save(&source).unwrap();
                let request = LegacyRelocationRequest {
                    source: admission_request("unused").destination,
                    target: PersistedDestinationIdentity {
                        library_root: "root-device-7-inode-11".into(),
                        relative_target: "llm/new/model".into(),
                    },
                    target_dir: tmp.path().join("new"),
                    model_type: None,
                    family: None,
                };
                let mut foreign = persisted("foreign");
                foreign.dest_dir = if at_target {
                    request.target_dir.clone()
                } else {
                    source.dest_dir.clone()
                };
                store.save(&foreign).unwrap();
                if quarantined {
                    store
                        .begin_lifecycle_quarantine(
                            &foreign,
                            LifecycleQuarantineDomain::Ambient,
                            true,
                        )
                        .unwrap();
                }
                assert!(store
                    .begin_legacy_relocation(&Uuid::new_v4().to_string(), &source, &request)
                    .is_err());
                let inventory = store.load_lifecycle_inventory_strict().unwrap();
                assert!(inventory.pending_relocations.is_empty());
                assert!(inventory
                    .downloads
                    .iter()
                    .any(|row| row.download_id == "source"));
            }
        }
    }

    #[test]
    fn legacy_relocation_publication_uncertainty_preserves_valid_restart_state() {
        for abort in [false, true] {
            for after_effect in [false, true] {
                let tmp = TempDir::new().unwrap();
                let store = DownloadPersistence::new(tmp.path());
                let source = persisted("relocation");
                store.save(&source).unwrap();
                let request = LegacyRelocationRequest {
                    source: admission_request("unused").destination,
                    target: PersistedDestinationIdentity {
                        library_root: "root-device-7-inode-11".into(),
                        relative_target: "llm/new/model".into(),
                    },
                    target_dir: tmp.path().join("new"),
                    model_type: None,
                    family: None,
                };
                let attempt = Uuid::new_v4().to_string();
                store
                    .begin_legacy_relocation(&attempt, &source, &request)
                    .unwrap()
                    .unwrap();
                let publisher = ScriptedPublisher::new([if after_effect {
                    ScriptedPublication::PublishedDurabilityUnknown
                } else {
                    ScriptedPublication::VisibilityUnknownBeforeEffect
                }]);
                let uncertain = store.with_test_publisher(Arc::new(publisher));
                let outcome = if abort {
                    uncertain.abort_legacy_relocation(&source.download_id, &attempt)
                } else {
                    uncertain.finish_legacy_relocation(&source.download_id, &attempt)
                }
                .unwrap()
                .unwrap();
                assert!(!matches!(outcome, AtomicPublication::Durable));
                let fresh = DownloadPersistence::new(tmp.path());
                let inventory = fresh.load_lifecycle_inventory_strict().unwrap();
                if after_effect {
                    assert!(inventory.pending_relocations.is_empty());
                    assert_eq!(
                        inventory.downloads[0].dest_dir,
                        if abort {
                            source.dest_dir
                        } else {
                            request.target_dir
                        }
                    );
                    assert!(fresh
                        .finish_legacy_relocation("relocation", &attempt)
                        .is_err());
                } else {
                    assert!(inventory.downloads.is_empty());
                    assert!(inventory.pending_relocations.contains_key("relocation"));
                }
            }
        }
    }

    #[test]
    fn uncertain_relocation_intent_never_exposes_source_as_resumable() {
        let tmp = TempDir::new().unwrap();
        let store = DownloadPersistence::new(tmp.path());
        let source = persisted("relocation");
        store.save(&source).unwrap();
        let request = LegacyRelocationRequest {
            source: admission_request("unused").destination,
            target: PersistedDestinationIdentity {
                library_root: "root-device-7-inode-11".into(),
                relative_target: "llm/new/model".into(),
            },
            target_dir: tmp.path().join("new"),
            model_type: None,
            family: None,
        };
        let attempt = Uuid::new_v4().to_string();
        let uncertain = store.with_test_publisher(Arc::new(ScriptedPublisher::new([
            ScriptedPublication::PublishedDurabilityUnknown,
        ])));
        assert!(matches!(
            uncertain
                .begin_legacy_relocation(&attempt, &source, &request)
                .unwrap()
                .unwrap(),
            AtomicPublication::PublishedDurabilityUnknown { .. }
        ));
        let fresh = DownloadPersistence::new(tmp.path());
        fresh.reconcile_lifecycle_inventory_strict().unwrap();
        assert!(fresh.load_all_strict().unwrap().is_empty());
        assert_eq!(
            fresh
                .load_lifecycle_inventory_strict()
                .unwrap()
                .pending_relocations["relocation"]
                .request,
            request
        );
        assert!(matches!(
            fresh
                .abort_legacy_relocation("relocation", &attempt)
                .unwrap()
                .unwrap(),
            AtomicPublication::Durable
        ));
        assert_eq!(
            fresh.load_all_strict().unwrap()[0].dest_dir,
            source.dest_dir
        );
    }

    #[test]
    fn legacy_relocation_parks_both_destinations_and_publishes_only_exact_target() {
        let tmp = TempDir::new().unwrap();
        let store = DownloadPersistence::new(tmp.path());
        let source = persisted("relocating");
        store.save(&source).unwrap();
        let request = LegacyRelocationRequest {
            source: admission_request("unused").destination,
            target: PersistedDestinationIdentity {
                library_root: "root-device-7-inode-11".into(),
                relative_target: "llm/new/model".into(),
            },
            target_dir: tmp.path().join("new"),
            model_type: Some("llm".into()),
            family: Some("new".into()),
        };
        let attempt = Uuid::new_v4().to_string();
        let mut stale = source.clone();
        stale.repo_id = "wrong/repo".into();
        assert!(store
            .begin_legacy_relocation(&attempt, &stale, &request)
            .is_err());
        assert!(matches!(
            store
                .begin_legacy_relocation(&attempt, &source, &request)
                .unwrap()
                .unwrap(),
            AtomicPublication::Durable
        ));
        let fresh = DownloadPersistence::new(tmp.path());
        assert!(fresh.load_all_strict().unwrap().is_empty());
        let inventory = fresh.load_lifecycle_inventory_strict().unwrap();
        assert_eq!(inventory.pending_relocations["relocating"].request, request);
        assert_eq!(
            inventory.pending_relocations["relocating"]
                .snapshot
                .dest_dir,
            source.dest_dir
        );
        for identity in [&request.source, &request.target] {
            let mut admission = admission_request("intruder");
            admission.destination = identity.clone();
            assert!(fresh
                .admit_download(&Uuid::new_v4().to_string(), &admission)
                .is_err());
        }
        assert!(fresh.save(&source).is_err());
        assert!(fresh.remove(&source.download_id).is_err());
        assert!(fresh
            .update_status(&source.download_id, DownloadStatus::Error)
            .is_err());
        assert!(fresh.revoke(&source.download_id).is_err());
        assert!(fresh
            .begin_lifecycle_quarantine(&source, LifecycleQuarantineDomain::Ambient, false)
            .is_err());
        assert!(fresh
            .finish_legacy_relocation(&source.download_id, &Uuid::new_v4().to_string())
            .is_err());
        assert!(matches!(
            fresh
                .finish_legacy_relocation(&source.download_id, &attempt)
                .unwrap()
                .unwrap(),
            AtomicPublication::Durable
        ));
        let after = DownloadPersistence::new(tmp.path())
            .load_all_strict()
            .unwrap();
        let mut expected = source;
        expected.dest_dir = request.target_dir;
        expected.download_request.model_type = request.model_type;
        expected.download_request.family = request.family.unwrap();
        assert!(persisted_download_matches(&after[0], &expected));
        assert!(fresh
            .finish_legacy_relocation(&expected.download_id, &attempt)
            .is_err());
    }

    #[test]
    fn admitted_status_update_requires_confirmed_exact_owner_and_preserves_request() {
        let tmp = TempDir::new().unwrap();
        let store = DownloadPersistence::new(tmp.path());
        let attempt = Uuid::new_v4().to_string();
        let request = admission_request("status");
        store.admit_download(&attempt, &request).unwrap();
        let before = store
            .load_lifecycle_inventory_strict()
            .unwrap()
            .queue_admissions["status"]
            .clone();
        assert!(!store
            .update_admitted_status("status", &Uuid::new_v4().to_string(), DownloadStatus::Error)
            .unwrap());
        let fresh = DownloadPersistence::new(tmp.path());
        assert!(!fresh
            .update_admitted_status("status", &attempt, DownloadStatus::Paused)
            .unwrap());
        assert!(!store
            .update_status("status", DownloadStatus::Error)
            .unwrap());
        assert!(store
            .update_admitted_status("status", &attempt, DownloadStatus::Paused)
            .unwrap());
        let inventory = store.load_lifecycle_inventory_strict().unwrap();
        let mut expected = request.snapshot.clone();
        expected.status = DownloadStatus::Paused;
        assert!(persisted_download_matches(
            &inventory.downloads[0],
            &expected
        ));
        assert_eq!(
            serde_json::to_value(&inventory.queue_admissions["status"]).unwrap(),
            serde_json::to_value(before).unwrap()
        );
        for terminal in [
            DownloadStatus::Cancelling,
            DownloadStatus::Completed,
            DownloadStatus::Cancelled,
        ] {
            assert!(store
                .update_admitted_status("status", &attempt, terminal)
                .is_err());
        }
        let uncertain = store
            .clone()
            .with_test_publisher(Arc::new(ScriptedPublisher::new([
                ScriptedPublication::PublishedDurabilityUnknown,
            ])));
        assert!(uncertain
            .update_admitted_status("status", &attempt, DownloadStatus::Error)
            .is_err());
        assert!(store
            .update_admitted_status("status", &attempt, DownloadStatus::Error)
            .unwrap());
        store
            .begin_lifecycle_quarantine(&request.snapshot, LifecycleQuarantineDomain::Ambient, true)
            .unwrap();
        assert!(!store
            .update_admitted_status("status", &attempt, DownloadStatus::Paused)
            .unwrap());
        store.verify_lifecycle_quarantine("status").unwrap();
        store.settle_queue_admission("status", &attempt).unwrap();
        assert!(!store
            .update_admitted_status("status", &attempt, DownloadStatus::Paused)
            .unwrap());
    }

    #[test]
    fn pending_intent_retry_preserves_sticky_failure_provenance() {
        let tmp = TempDir::new().unwrap();
        let store = DownloadPersistence::new(tmp.path()).with_test_publisher(Arc::new(
            ScriptedPublisher::new([ScriptedPublication::PublishedDurabilityUnknown]),
        ));
        let snapshot = persisted("sticky-intent");
        assert!(store
            .begin_lifecycle_quarantine(&snapshot, LifecycleQuarantineDomain::Ambient, true)
            .is_err());
        store
            .begin_lifecycle_quarantine(&snapshot, LifecycleQuarantineDomain::Ambient, false)
            .unwrap();
        let inventory = store.load_lifecycle_inventory_strict().unwrap();
        assert!(inventory.quarantines["sticky-intent"].sticky_failure);
        assert_eq!(
            inventory.quarantines["sticky-intent"].snapshot.status,
            DownloadStatus::Error
        );
    }

    #[test]
    fn released_attempt_cannot_authorize_a_different_download() {
        let tmp = TempDir::new().unwrap();
        let store = DownloadPersistence::new(tmp.path());
        let attempt = Uuid::new_v4().to_string();
        store
            .admit_download(&attempt, &admission_request("first"))
            .unwrap();
        store.settle_queue_admission("first", &attempt).unwrap();
        assert!(store
            .admit_download(&attempt, &admission_request("second"))
            .is_err());
        assert!(store.load_all_strict().unwrap().is_empty());
    }

    #[test]
    fn pending_and_sticky_quarantine_retries_require_a_confirmed_barrier() {
        for transition in ["pending", "sticky"] {
            let tmp = TempDir::new().unwrap();
            let store = DownloadPersistence::new(tmp.path());
            let snapshot = persisted("cleanup");
            if transition == "sticky" {
                store
                    .begin_lifecycle_quarantine(
                        &snapshot,
                        LifecycleQuarantineDomain::Ambient,
                        false,
                    )
                    .unwrap();
            }
            let script = if transition == "pending" {
                vec![
                    ScriptedPublication::Durable,
                    ScriptedPublication::PublishedDurabilityUnknown,
                    ScriptedPublication::NotPublished,
                ]
            } else {
                vec![
                    ScriptedPublication::PublishedDurabilityUnknown,
                    ScriptedPublication::NotPublished,
                ]
            };
            let uncertain = store.with_test_publisher(Arc::new(ScriptedPublisher::new(script)));
            for _ in 0..2 {
                let failed = if transition == "pending" {
                    uncertain
                        .begin_lifecycle_quarantine(
                            &snapshot,
                            LifecycleQuarantineDomain::Ambient,
                            false,
                        )
                        .is_err()
                } else {
                    uncertain
                        .mark_lifecycle_quarantine_failed("cleanup")
                        .is_err()
                };
                assert!(
                    failed,
                    "{transition} must not infer durability from persisted bytes"
                );
            }
        }
    }

    #[test]
    fn conflicting_save_is_rejected_before_overwriting_a_hidden_admission() {
        let tmp = TempDir::new().unwrap();
        let store = DownloadPersistence::new(tmp.path()).with_test_publisher(Arc::new(
            ScriptedPublisher::new([ScriptedPublication::PublishedDurabilityUnknown]),
        ));
        let request = admission_request("hidden");
        assert!(matches!(
            store
                .admit_download(&Uuid::new_v4().to_string(), &request)
                .unwrap(),
            DownloadAdmissionTransition::PublishedDurabilityUnknown { .. }
        ));
        assert!(store.save(&request.snapshot).is_err());
        let fresh = DownloadPersistence::new(tmp.path())
            .load_lifecycle_inventory_strict()
            .unwrap();
        assert!(fresh.hidden_admissions.contains_key("hidden"));
        assert!(fresh.downloads.is_empty());
    }

    #[test]
    fn exact_release_survives_restart_without_orphaning_a_cross_domain_follower() {
        let tmp = TempDir::new().unwrap();
        let store = DownloadPersistence::new(tmp.path());
        let first = Uuid::new_v4().to_string();
        store
            .admit_download(&first, &admission_request("first"))
            .unwrap();
        let mut request = admission_request("second");
        request.domain = DownloadAdmissionDomain::Recovery;
        store
            .admit_download(&Uuid::new_v4().to_string(), &request)
            .unwrap();
        assert!(store
            .settle_queue_admission("first", &Uuid::new_v4().to_string())
            .is_err());
        let uncertain = store.with_test_publisher(Arc::new(ScriptedPublisher::new([
            ScriptedPublication::PublishedDurabilityUnknown,
            ScriptedPublication::NotPublished,
        ])));
        assert!(uncertain.settle_queue_admission("first", &first).is_err());
        assert!(uncertain
            .load_all_strict()
            .unwrap()
            .iter()
            .all(|snapshot| snapshot.download_id != "first"));
        assert!(uncertain.settle_queue_admission("first", &first).is_err());
        let fresh = DownloadPersistence::new(tmp.path());
        assert!(fresh.settle_queue_admission("first", &first).unwrap());
        fresh.reconcile_lifecycle_inventory_strict().unwrap();
        let inventory = fresh.load_lifecycle_inventory_strict().unwrap();
        assert_eq!(inventory.downloads.len(), 1);
        assert_eq!(inventory.downloads[0].download_id, "second");
        assert_eq!(inventory.queue_admissions["second"].position.ordinal, 1);
        assert_eq!(
            inventory.queue_admissions["second"]
                .position
                .predecessor
                .as_ref()
                .unwrap()
                .admission_attempt_id,
            first
        );
        assert!(fresh.settle_queue_admission("first", &first).unwrap());
        assert!(!fresh.settle_queue_admission("absent", &first).unwrap());
    }

    #[test]
    fn cleanup_retry_and_restart_require_a_confirmed_barrier() {
        let tmp = TempDir::new().unwrap();
        let store = DownloadPersistence::new(tmp.path());
        store
            .begin_lifecycle_quarantine(
                &persisted("cleanup"),
                LifecycleQuarantineDomain::Ambient,
                true,
            )
            .unwrap();
        let uncertain = store.with_test_publisher(Arc::new(ScriptedPublisher::new([
            ScriptedPublication::Durable,
            ScriptedPublication::PublishedDurabilityUnknown,
            ScriptedPublication::NotPublished,
        ])));
        assert!(uncertain.verify_lifecycle_quarantine("cleanup").is_err());
        assert!(uncertain.verify_lifecycle_quarantine("cleanup").is_err());
        let fresh = DownloadPersistence::new(tmp.path());
        assert_eq!(
            fresh.load_lifecycle_inventory_strict().unwrap().quarantines["cleanup"].disposition,
            LifecycleCleanupDisposition::Pending
        );
        fresh.reconcile_lifecycle_inventory_strict().unwrap();
        assert_eq!(
            fresh.load_lifecycle_inventory_strict().unwrap().quarantines["cleanup"].disposition,
            LifecycleCleanupDisposition::Verified
        );
    }

    #[test]
    fn legacy_mutations_cannot_release_or_relocate_queue_owned_downloads() {
        for operation in [
            "quarantine_drift",
            "save",
            "remove",
            "revoke",
            "relocate",
            "clean_quarantine",
        ] {
            let tmp = TempDir::new().unwrap();
            let store = DownloadPersistence::new(tmp.path());
            let request = admission_request("queued");
            store
                .admit_download(&Uuid::new_v4().to_string(), &request)
                .unwrap();
            let rejected = match operation {
                "quarantine_drift" => {
                    let mut stale = request.snapshot.clone();
                    stale.dest_dir = tmp.path().join("other");
                    store
                        .begin_lifecycle_quarantine(
                            &stale,
                            LifecycleQuarantineDomain::Ambient,
                            false,
                        )
                        .is_err()
                }
                "save" => {
                    let mut stale = request.snapshot.clone();
                    stale.dest_dir = tmp.path().join("other");
                    store.save(&stale).is_err()
                }
                "remove" => store.remove("queued").is_err(),
                "revoke" => store.revoke("queued").is_err(),
                "relocate" => store
                    .begin_legacy_relocation(
                        &Uuid::new_v4().to_string(),
                        &request.snapshot,
                        &legacy_relocation_request(tmp.path().join("other")),
                    )
                    .is_err(),
                _ => {
                    store
                        .begin_lifecycle_quarantine(
                            &request.snapshot,
                            LifecycleQuarantineDomain::Ambient,
                            false,
                        )
                        .unwrap();
                    store.remove_clean_lifecycle_quarantine("queued").is_err()
                }
            };
            assert!(
                rejected,
                "{operation} must require an exact queue transition"
            );
            let fresh = DownloadPersistence::new(tmp.path());
            fresh.reconcile_lifecycle_inventory_strict().unwrap();
            let inventory = fresh.load_lifecycle_inventory_strict().unwrap();
            assert!(inventory.queue_admissions.contains_key("queued"));
            if operation != "clean_quarantine" {
                assert_eq!(inventory.downloads[0].dest_dir, request.snapshot.dest_dir);
            }
        }
    }

    #[test]
    fn cross_domain_admissions_share_the_physical_destination_queue() {
        let tmp = TempDir::new().unwrap();
        let store = DownloadPersistence::new(tmp.path());
        let first = Uuid::new_v4().to_string();
        store
            .admit_download(&first, &admission_request("first"))
            .unwrap();
        let mut request = admission_request("second");
        request.domain = DownloadAdmissionDomain::Recovery;
        let outcome = store
            .admit_download(&Uuid::new_v4().to_string(), &request)
            .unwrap();
        let DownloadAdmissionTransition::Durable { admission, .. } = outcome else {
            panic!("second admission must be durable");
        };
        assert_eq!(admission.position.ordinal, 1);
        assert_eq!(
            admission.position.predecessor,
            Some(QueuePredecessor {
                download_id: "first".into(),
                admission_attempt_id: first,
            })
        );
    }

    #[test]
    fn admission_requires_two_confirmed_publications_before_becoming_public() {
        let tmp = TempDir::new().unwrap();
        let publisher = Arc::new(ScriptedPublisher::new([
            ScriptedPublication::Durable,
            ScriptedPublication::PublishedDurabilityUnknown,
        ]));
        let store = DownloadPersistence::new(tmp.path()).with_test_publisher(publisher);
        let attempt_id = Uuid::new_v4().to_string();
        let request = admission_request("dl-admission-unknown");

        let outcome = store
            .admit_download(&attempt_id, &request)
            .expect("store validation must succeed");

        assert!(matches!(
            outcome,
            DownloadAdmissionTransition::PublishedDurabilityUnknown {
                phase: DownloadAdmissionPhase::Confirmation,
                ..
            }
        ));
        let inventory = DownloadPersistence::new(tmp.path())
            .load_lifecycle_inventory_strict()
            .unwrap();
        assert!(inventory.downloads.is_empty());
        assert!(inventory
            .hidden_admissions
            .contains_key("dl-admission-unknown"));
        assert!(inventory.queue_admissions.is_empty());

        let retry_store = DownloadPersistence::new(tmp.path());
        let retry = retry_store.admit_download(&attempt_id, &request).unwrap();
        let DownloadAdmissionTransition::Durable { admission, .. } = retry else {
            panic!("matching retry must confirm the same durable admission");
        };
        assert_eq!(admission.position.ordinal, 0);
        assert_eq!(admission.position.predecessor, None);
        let inventory = retry_store.load_lifecycle_inventory_strict().unwrap();
        assert_eq!(inventory.downloads.len(), 1);
        assert_eq!(inventory.downloads[0].download_id, "dl-admission-unknown");
        assert!(inventory.hidden_admissions.is_empty());
        assert_eq!(
            inventory.queue_admissions["dl-admission-unknown"].attempt_id,
            attempt_id
        );
        let restarted = DownloadPersistence::new(tmp.path());
        assert!(restarted.load_all_strict().unwrap().is_empty());
        let failed = restarted
            .clone()
            .with_test_publisher(Arc::new(ScriptedPublisher::new([
                ScriptedPublication::PublishedDurabilityUnknown,
            ])));
        assert!(failed.reconcile_lifecycle_inventory_strict().is_err());
        assert!(restarted.load_all_strict().unwrap().is_empty());
        restarted.reconcile_lifecycle_inventory_strict().unwrap();
        assert_eq!(restarted.load_all_strict().unwrap().len(), 1);
    }
}
