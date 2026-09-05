#![deny(unsafe_code)]

use crate::{ModelRecord, PumasError, Result};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

const TOKEN_DOMAIN: &[u8] = b"pumas-download-recovery\0v1";
const TOKEN_PREFIX: &str = "v1:";
const TOKEN_HEX_BYTES: usize = 64;
const MAX_IDENTIFIER_BYTES: usize = 4 * 1024;
const MAX_COLLECTION_ITEMS: usize = 512;
const MAX_HF_REPO_ID_BYTES: usize = 96;
const MAX_PORTABLE_PATH_COMPONENT_BYTES: usize = 255;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct FilesystemIdentity {
    volume: u64,
    file: u64,
}

#[cfg(unix)]
fn filesystem_identity(metadata: &std::fs::Metadata) -> Option<FilesystemIdentity> {
    use std::os::unix::fs::MetadataExt;
    Some(FilesystemIdentity {
        volume: metadata.dev(),
        file: metadata.ino(),
    })
}

#[cfg(windows)]
fn filesystem_identity(metadata: &std::fs::Metadata) -> Option<FilesystemIdentity> {
    use std::os::windows::fs::MetadataExt;
    Some(FilesystemIdentity {
        volume: u64::from(metadata.volume_serial_number()?),
        file: metadata.file_index()?,
    })
}

#[cfg(not(any(unix, windows)))]
fn filesystem_identity(_metadata: &std::fs::Metadata) -> Option<FilesystemIdentity> {
    None
}

/// Opaque collision-resistant fingerprint of one recovery-relevant model state.
///
/// This is a stale-state precondition, not an authentication credential. The
/// recovery target and repository are always re-resolved by the core.
#[derive(Clone, PartialEq, Eq)]
pub struct DownloadRecoveryToken(String);

impl DownloadRecoveryToken {
    pub fn parse(value: &str) -> Option<Self> {
        let digest = value.strip_prefix(TOKEN_PREFIX)?;
        (digest.len() == TOKEN_HEX_BYTES
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)))
        .then(|| Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Validated platform-neutral model ID accepted by the recovery action.
#[derive(Clone, PartialEq, Eq)]
pub struct DownloadRecoveryModelId(String);

impl DownloadRecoveryModelId {
    pub fn parse(value: &str) -> Option<Self> {
        is_portable_relative_path(value).then(|| Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Producer-issued display/action precondition for one partial model.
#[derive(Clone)]
pub struct DownloadRecoveryTicket {
    token: DownloadRecoveryToken,
    repo_id: String,
    selected_artifact_id: Option<String>,
    selected_artifact_files: Vec<String>,
    selected_artifact_quant: Option<String>,
}

impl DownloadRecoveryTicket {
    pub fn token(&self) -> &str {
        self.token.as_str()
    }

    pub fn repo_id(&self) -> &str {
        &self.repo_id
    }

    pub fn selected_artifact_id(&self) -> Option<&str> {
        self.selected_artifact_id.as_deref()
    }

    pub fn selected_artifact_files(&self) -> &[String] {
        &self.selected_artifact_files
    }

    pub fn selected_artifact_quant(&self) -> Option<&str> {
        self.selected_artifact_quant.as_deref()
    }
}

#[derive(Clone)]
pub(crate) struct VerifiedDownloadRecovery {
    pub(crate) destination: DownloadRecoveryDestination,
    pub(crate) repo_id: String,
    pub(crate) files: Vec<String>,
}

/// Held library-root authority for one verified recovery destination.
///
/// Recovery file operations remain relative to this capability instead of
/// regaining ambient authority from the displayed absolute path.
#[derive(Clone)]
pub(crate) struct DownloadRecoveryDestination {
    authority: Arc<RecoveryRoot>,
    model_relative: PathBuf,
    display_path: PathBuf,
    held: Arc<OnceLock<HeldDestination>>,
    creation_anchor: Arc<CreationAnchor>,
    file_parents: Arc<Mutex<BTreeMap<PathBuf, Arc<HeldDestination>>>>,
}

impl super::partial_download::PartialDownloadFiles for DownloadRecoveryDestination {
    fn file_len(&self, filename: &str) -> Result<Option<u64>> {
        Ok(self.file_len(filename)?)
    }
    fn part_len(&self, filename: &str) -> Result<Option<u64>> {
        Ok(self.part_len(filename)?)
    }
    fn rename_part_to_file(&self, filename: &str) -> Result<()> {
        Ok(self.rename_part_to_file(filename)?)
    }
    fn remove_part(&self, filename: &str) -> Result<()> {
        Ok(self.remove_part(filename)?)
    }
    fn remove_marker(&self) -> Result<()> {
        Ok(self.remove_marker()?)
    }
}

struct CreationAnchor {
    directory: Dir,
    relative: PathBuf,
    tail: PathBuf,
    identity: FilesystemIdentity,
}

impl CreationAnchor {
    fn capture(root: &Dir, target: &Path) -> io::Result<Self> {
        for prefix in target.ancestors() {
            match open_directory_chain(root, prefix, false) {
                Ok(directory) => {
                    return Ok(Self {
                        identity: directory_identity(&directory)?,
                        directory,
                        relative: prefix.to_path_buf(),
                        tail: target
                            .strip_prefix(prefix)
                            .map_err(|_| invalid_capability_path())?
                            .to_path_buf(),
                    });
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
        Err(invalid_capability_path())
    }
}

struct HeldDestination {
    directory: Dir,
    identity: FilesystemIdentity,
}

/// Equality information only. Effect authority lives in the held capability.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct DestinationIdentity {
    root: FilesystemIdentity,
    relative: String,
}

impl DestinationIdentity {
    pub(crate) fn persisted(&self) -> super::download_store::PersistedDestinationIdentity {
        super::download_store::PersistedDestinationIdentity {
            library_root: format!("unix:{}:{}", self.root.volume, self.root.file),
            relative_target: self.relative.clone(),
        }
    }
}

/// One configured root opened by the composition owner after directory setup.
#[derive(Clone)]
pub(crate) struct DownloadDestinationRoot(Arc<RecoveryRoot>);

impl DownloadDestinationRoot {
    pub(crate) fn open(path: &Path) -> Result<Self> {
        #[cfg(not(unix))]
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Download destination authority requires Unix",
        )
        .into());
        #[cfg(unix)]
        RecoveryRoot::open(path)?
            .map(|root| Self(Arc::new(root)))
            .ok_or_else(|| invalid_capability_path().into())
    }

    pub(crate) fn resolve(&self, path: &Path) -> io::Result<DownloadRecoveryDestination> {
        self.0.require_current()?;
        // Find a prefix which is the configured root itself. Missing descendants
        // never determine identity, and symlinks below this prefix are rejected.
        let mut relative = (!path.is_absolute()).then(|| path.to_path_buf());
        for ancestor in path.ancestors().skip(1).filter(|_| path.is_absolute()) {
            if let Ok(metadata) = std::fs::metadata(ancestor) {
                if filesystem_identity(&metadata) == Some(self.0.root_identity) {
                    relative = path.strip_prefix(ancestor).ok().map(Path::to_path_buf);
                    break;
                }
            }
        }
        let relative = relative.ok_or_else(invalid_capability_path)?;
        let text = relative.to_str().ok_or_else(invalid_capability_path)?;
        if !is_portable_relative_path(text) {
            return Err(invalid_capability_path());
        }
        let destination = DownloadRecoveryDestination {
            authority: self.0.clone(),
            display_path: self.0.root_canonical_path.join(&relative),
            creation_anchor: Arc::new(CreationAnchor::capture(&self.0.root, &relative)?),
            model_relative: relative,
            held: Arc::new(OnceLock::new()),
            file_parents: Arc::new(Mutex::new(BTreeMap::new())),
        };
        match destination.directory(false) {
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                destination.require_directory_chain(&destination.model_relative, true)?;
            }
            Err(error) => return Err(error),
        }
        self.0.require_current()?;
        Ok(destination)
    }
}

struct RecoveryRoot {
    root: Dir,
    root_source_path: PathBuf,
    root_canonical_path: PathBuf,
    root_identity: FilesystemIdentity,
}

impl RecoveryRoot {
    fn open(library_root: &Path) -> Result<Option<Self>> {
        let root =
            Dir::open_ambient_dir(library_root, ambient_authority()).map_err(PumasError::from)?;
        let held_metadata = root
            .try_clone()
            .and_then(|root| root.into_std_file().metadata())
            .map_err(PumasError::from)?;
        let Some(root_identity) = filesystem_identity(&held_metadata) else {
            return Ok(None);
        };
        let root_canonical_path = std::fs::canonicalize(library_root).map_err(PumasError::from)?;
        let authority = Self {
            root,
            root_source_path: library_root.to_path_buf(),
            root_canonical_path,
            root_identity,
        };
        if authority.require_current().is_err() {
            return Ok(None);
        }
        Ok(Some(authority))
    }

    fn destination_for(
        self: &Arc<Self>,
        record: &ModelRecord,
    ) -> Option<DownloadRecoveryDestination> {
        if !is_portable_relative_path(&record.id) || self.require_current().is_err() {
            return None;
        }
        let model_relative = PathBuf::from(&record.id);
        let display_path = self.root_canonical_path.join(&model_relative);
        if Path::new(&record.path) != display_path {
            return None;
        }
        let model_metadata = self.root.symlink_metadata(&model_relative).ok()?;
        if !model_metadata.is_dir() || model_metadata.is_symlink() {
            return None;
        }
        if self.root.canonicalize(&model_relative).ok()? != model_relative
            || self.require_current().is_err()
        {
            return None;
        }
        Some(DownloadRecoveryDestination {
            authority: self.clone(),
            creation_anchor: Arc::new(CreationAnchor::capture(&self.root, &model_relative).ok()?),
            model_relative,
            display_path,
            held: Arc::new(OnceLock::new()),
            file_parents: Arc::new(Mutex::new(BTreeMap::new())),
        })
    }

    fn require_current(&self) -> io::Result<()> {
        let canonical = std::fs::canonicalize(&self.root_source_path)?;
        if canonical != self.root_canonical_path {
            return Err(invalid_capability_path());
        }
        let current = std::fs::metadata(&canonical)?;
        if filesystem_identity(&current) != Some(self.root_identity) {
            return Err(invalid_capability_path());
        }
        let held = self.root.try_clone()?.into_std_file().metadata()?;
        if filesystem_identity(&held) != Some(self.root_identity) {
            return Err(invalid_capability_path());
        }
        Ok(())
    }
}

impl DownloadRecoveryDestination {
    pub(crate) fn identity(&self) -> DestinationIdentity {
        DestinationIdentity {
            root: self.authority.root_identity,
            relative: self.model_relative.to_string_lossy().into_owned(),
        }
    }

    fn directory(&self, create: bool) -> io::Result<Dir> {
        self.directory_if_present(create)?.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "Download destination does not exist",
            )
        })
    }

    /// Absence is legitimate only before this capability has held a target.
    /// Root and creation-anchor failures never become successful empty cleanup.
    fn directory_if_present(&self, create: bool) -> io::Result<Option<Dir>> {
        self.authority.require_current()?;
        let anchor = &self.creation_anchor;
        if directory_identity(&open_directory_chain(
            &self.authority.root,
            &anchor.relative,
            false,
        )?)? != anchor.identity
        {
            return Err(invalid_capability_path());
        }
        let directory = match open_directory_chain(
            &anchor.directory,
            &anchor.tail,
            create && self.held.get().is_none(),
        ) {
            Ok(directory) => directory,
            Err(error)
                if !create
                    && self.held.get().is_none()
                    && error.kind() == io::ErrorKind::NotFound =>
            {
                self.authority.require_current()?;
                return Ok(None);
            }
            Err(error) => return Err(error),
        };
        let identity = directory_identity(&directory)?;
        if let Some(held) = self.held.get() {
            if held.identity != identity {
                return Err(invalid_capability_path());
            }
        } else {
            let _ = self.held.set(HeldDestination {
                directory: directory.try_clone()?,
                identity,
            });
        }
        let held = self.held.get().ok_or_else(invalid_capability_path)?;
        if held.identity != identity {
            return Err(invalid_capability_path());
        }
        self.authority.require_current()?;
        held.directory.try_clone().map(Some)
    }

    pub(crate) fn prepare(&self) -> io::Result<()> {
        self.directory(true).map(|_| ())
    }

    fn file_parent(&self, file: &str, create: bool) -> io::Result<(Dir, String)> {
        self.file_parent_if_present(file, create)?.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "Download file parent does not exist",
            )
        })
    }

    fn file_parent_if_present(
        &self,
        file: &str,
        create: bool,
    ) -> io::Result<Option<(Dir, String)>> {
        if !is_portable_relative_path(file) {
            return Err(invalid_capability_path());
        }
        let path = Path::new(file);
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(invalid_capability_path)?;
        let Some(mut directory) = self.directory_if_present(false)? else {
            return Ok(None);
        };
        let mut relative = PathBuf::new();
        for component in path.parent().unwrap_or(Path::new("")).components() {
            let Component::Normal(component) = component else {
                return Err(invalid_capability_path());
            };
            relative.push(component);
            let known = {
                self.file_parents
                    .lock()
                    .map_err(|_| io::Error::other("Download parent authority lock poisoned"))?
                    .get(&relative)
                    .cloned()
            };
            let next = match open_directory_chain(
                &directory,
                Path::new(component),
                create && known.is_none(),
            ) {
                Ok(directory) => directory,
                Err(error)
                    if !create && known.is_none() && error.kind() == io::ErrorKind::NotFound =>
                {
                    return Ok(None);
                }
                Err(error) => return Err(error),
            };
            let identity = directory_identity(&next)?;
            let candidate = Arc::new(HeldDestination {
                directory: next,
                identity,
            });
            let held = {
                let mut parents = self
                    .file_parents
                    .lock()
                    .map_err(|_| io::Error::other("Download parent authority lock poisoned"))?;
                parents.entry(relative.clone()).or_insert(candidate).clone()
            };
            if held.identity != identity {
                return Err(invalid_capability_path());
            }
            directory = held.directory.try_clone()?;
        }
        Ok(Some((directory, name.to_owned())))
    }

    /// Publish the unchanged object schema through held directory authority.
    pub(crate) fn write_marker(&self, marker: &Value) -> crate::metadata::AtomicPublishResult {
        if !marker.is_object() {
            return Err(Box::new(crate::metadata::AtomicPublishFailure {
                stage: crate::metadata::AtomicPublishStage::Serialization,
                kind: crate::metadata::AtomicPublishFailureKind::InvalidData,
                error: PumasError::Other("Download marker must be a JSON object".into()),
                cleanup: crate::metadata::StagingCleanup::NotRequired,
            }));
        }
        let admitted = (|| -> Result<crate::metadata::AtomicJsonTarget> {
            let parent = self.directory(false)?;
            let expected = directory_identity(&parent)?;
            let destination = self.clone();
            crate::metadata::AtomicJsonTarget::from_capability(
                parent,
                std::ffi::OsStr::new(".pumas_download"),
                self.display_path.join(".pumas_download"),
                move || Ok(directory_identity(&destination.directory(false)?)? == expected),
            )
        })();
        match admitted {
            Ok(target) => target.publish_json(marker),
            Err(error) => Err(Box::new(crate::metadata::AtomicPublishFailure {
                stage: crate::metadata::AtomicPublishStage::TargetAdmission,
                kind: crate::metadata::AtomicPublishFailureKind::TargetUnavailable,
                error,
                cleanup: crate::metadata::StagingCleanup::NotRequired,
            })),
        }
    }
    pub(crate) fn display_path(&self) -> &Path {
        &self.display_path
    }

    #[cfg(test)]
    pub(crate) fn authority_strong_count(&self) -> usize {
        Arc::strong_count(&self.authority)
    }

    pub(crate) fn preflight(&self, files: &[String]) -> io::Result<()> {
        self.authority.require_current()?;
        self.require_directory_chain(&self.model_relative, false)?;
        for file in files {
            let file = Path::new(file);
            let relative = self.model_relative.join(file);
            if let Some(parent) = relative.parent() {
                self.require_directory_chain(parent, true)?;
            }
            self.require_regular_or_missing(&relative)?;
            self.require_regular_or_missing(&self.part_relative(file))?;
        }
        self.authority.require_current()?;
        Ok(())
    }

    pub(crate) fn create_parent(&self, file: &str) -> io::Result<()> {
        self.file_parent(file, true).map(|_| ())
    }

    pub(crate) fn file_len(&self, file: &str) -> io::Result<Option<u64>> {
        self.regular_file_len(&self.model_relative.join(file))
    }

    pub(crate) fn part_len(&self, file: &str) -> io::Result<Option<u64>> {
        self.regular_file_len(&self.part_relative(Path::new(file)))
    }

    pub(crate) fn open_part(&self, file: &str, append: bool) -> io::Result<std::fs::File> {
        let (parent, name) = self.file_parent(file, true)?;
        let name = format!(
            "{name}{}",
            crate::config::NetworkConfig::DOWNLOAD_TEMP_SUFFIX
        );
        let mut options = OpenOptions::new();
        #[cfg(unix)]
        {
            use cap_std::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NOFOLLOW);
        }
        if append {
            options.append(true);
        } else {
            options.write(true).create(true).truncate(true);
        }
        parent.open_with(name, &options).map(|file| file.into_std())
    }

    pub(crate) fn remove_part(&self, file: &str) -> io::Result<()> {
        let Some((parent, name)) = self.file_parent_if_present(file, false)? else {
            return Ok(());
        };
        let name = format!(
            "{name}{}",
            crate::config::NetworkConfig::DOWNLOAD_TEMP_SUFFIX
        );
        match parent.remove_file(name) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }

    pub(crate) fn rename_part_to_file(&self, file: &str) -> io::Result<()> {
        let (parent, name) = self.file_parent(file, false)?;
        let part = format!(
            "{name}{}",
            crate::config::NetworkConfig::DOWNLOAD_TEMP_SUFFIX
        );
        parent.rename(part, &parent, name)
    }

    pub(crate) fn remove_marker(&self) -> io::Result<()> {
        let Some(directory) = self.directory_if_present(false)? else {
            return Ok(());
        };
        match directory.remove_file(".pumas_download") {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }

    fn regular_file_len(&self, relative: &Path) -> io::Result<Option<u64>> {
        let file = relative
            .strip_prefix(&self.model_relative)
            .map_err(|_| invalid_capability_path())?;
        let file = file.to_str().ok_or_else(invalid_capability_path)?;
        let Some((parent, name)) = self.file_parent_if_present(file, false)? else {
            return Ok(None);
        };
        match parent.symlink_metadata(name) {
            Ok(metadata) if metadata.is_file() => Ok(Some(metadata.len())),
            Ok(_) => Err(invalid_capability_path()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn require_regular_or_missing(&self, relative: &Path) -> io::Result<()> {
        self.regular_file_len(relative).map(|_| ())
    }

    fn require_directory_chain(&self, relative: &Path, allow_missing_tail: bool) -> io::Result<()> {
        let mut current = PathBuf::new();
        let mut missing = false;
        for component in relative.components() {
            let Component::Normal(component) = component else {
                return Err(invalid_capability_path());
            };
            current.push(component);
            if missing {
                continue;
            }
            match self.authority.root.symlink_metadata(&current) {
                Ok(metadata) if metadata.is_dir() => {}
                Ok(_) => return Err(invalid_capability_path()),
                Err(error) if allow_missing_tail && error.kind() == io::ErrorKind::NotFound => {
                    missing = true;
                }
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }

    fn part_relative(&self, file: &Path) -> PathBuf {
        let mut relative = self.model_relative.join(file).into_os_string();
        relative.push(crate::config::NetworkConfig::DOWNLOAD_TEMP_SUFFIX);
        PathBuf::from(relative)
    }
}

fn directory_identity(directory: &Dir) -> io::Result<FilesystemIdentity> {
    filesystem_identity(&directory.try_clone()?.into_std_file().metadata()?)
        .ok_or_else(invalid_capability_path)
}

/// Walk one component at a time without following symlinks. Each next operation
/// is anchored to the held preceding directory, including missing-tail creation.
fn open_directory_chain(root: &Dir, relative: &Path, create: bool) -> io::Result<Dir> {
    #[cfg(not(unix))]
    {
        let _ = (root, relative, create);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "No-follow directory authority unavailable",
        ))
    }
    #[cfg(unix)]
    {
        use cap_std::fs::OpenOptionsExt;
        let mut directory = root.try_clone()?;
        for component in relative.components() {
            let Component::Normal(name) = component else {
                return Err(invalid_capability_path());
            };
            let mut options = OpenOptions::new();
            options
                .read(true)
                .custom_flags(libc::O_NOFOLLOW | libc::O_DIRECTORY);
            let file = match directory.open_with(name, &options) {
                Ok(file) => file,
                Err(error) if create && error.kind() == io::ErrorKind::NotFound => {
                    match directory.create_dir(name) {
                        Ok(()) => {}
                        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                        Err(error) => return Err(error),
                    }
                    directory.open_with(name, &options)?
                }
                Err(error) => return Err(error),
            };
            if create {
                // Also sync existing entries: they may come from a previous
                // attempt whose directory-link durability was uncertain.
                file.sync_all()?;
                directory.open(".")?.sync_all()?;
            }
            directory = Dir::from_std_file(file.into_std());
        }
        Ok(directory)
    }
}

pub(crate) enum DownloadRecoveryVerification {
    Complete,
    Unavailable,
    Stale,
    Verified(VerifiedDownloadRecovery),
}

/// Atomic result of admitting one producer-verified recovery action.
pub(crate) enum RecoveryDownloadAdmission {
    Recovered {
        download_id: String,
    },
    Resumed {
        download_id: String,
    },
    Attached {
        download_id: String,
        status: crate::models::DownloadStatus,
    },
    AlreadyCompleted {
        download_id: String,
    },
    AlreadyCancelled {
        download_id: String,
    },
    ContextMismatch,
    BoundFilesUnavailable,
    CapabilityUnavailable,
}

#[derive(Clone)]
struct RecoverySnapshot {
    model_id: String,
    canonical_model_dir: String,
    root_identity: FilesystemIdentity,
    destination: DownloadRecoveryDestination,
    repo_id: String,
    selected_artifact_id: Option<String>,
    selected_artifact_files: Vec<String>,
    selected_artifact_quant: Option<String>,
}

impl RecoverySnapshot {
    fn token(&self) -> DownloadRecoveryToken {
        let mut hasher = blake3::Hasher::new();
        hasher.update(TOKEN_DOMAIN);
        hash_text(&mut hasher, &self.model_id);
        hash_text(&mut hasher, &self.canonical_model_dir);
        hasher.update(&self.root_identity.volume.to_be_bytes());
        hasher.update(&self.root_identity.file.to_be_bytes());
        hash_text(&mut hasher, &self.repo_id);
        hash_optional_text(&mut hasher, self.selected_artifact_id.as_deref());
        hash_optional_text(&mut hasher, self.selected_artifact_quant.as_deref());
        hasher.update(&(self.selected_artifact_files.len() as u64).to_be_bytes());
        for file in &self.selected_artifact_files {
            hash_text(&mut hasher, file);
        }
        DownloadRecoveryToken(format!("{TOKEN_PREFIX}{}", hasher.finalize().to_hex()))
    }

    fn into_ticket(self) -> DownloadRecoveryTicket {
        let token = self.token();
        DownloadRecoveryTicket {
            token,
            repo_id: self.repo_id,
            selected_artifact_id: self.selected_artifact_id,
            selected_artifact_files: self.selected_artifact_files,
            selected_artifact_quant: self.selected_artifact_quant,
        }
    }
}

fn hash_text(hasher: &mut blake3::Hasher, value: &str) {
    hasher.update(&(value.len() as u64).to_be_bytes());
    hasher.update(value.as_bytes());
}

fn hash_optional_text(hasher: &mut blake3::Hasher, value: Option<&str>) {
    match value {
        Some(value) => {
            hasher.update(&[1]);
            hash_text(hasher, value);
        }
        None => {
            hasher.update(&[0]);
        }
    }
}

/// Issue a recovery ticket from one core-owned projected model record.
///
/// Filesystem-ineligible partial records remain displayable and return no
/// ticket. Malformed recovery metadata is rejected when recovery is present.
pub fn issue_download_recovery_ticket(
    library_root: &Path,
    record: &ModelRecord,
) -> Result<Option<DownloadRecoveryTicket>> {
    Ok(recovery_snapshot(library_root, record)?.map(RecoverySnapshot::into_ticket))
}

pub(crate) fn verify_download_recovery_ticket(
    library_root: &Path,
    record: &ModelRecord,
    token: &DownloadRecoveryToken,
) -> Result<DownloadRecoveryVerification> {
    let incomplete = record
        .metadata
        .as_object()
        .and_then(|metadata| metadata.get("download_incomplete"))
        .and_then(Value::as_bool)
        .ok_or_else(invalid_recovery_metadata)?;
    if !incomplete {
        return Ok(DownloadRecoveryVerification::Complete);
    }
    let Some(snapshot) = recovery_snapshot(library_root, record)? else {
        return Ok(DownloadRecoveryVerification::Unavailable);
    };
    if snapshot.token() != *token {
        return Ok(DownloadRecoveryVerification::Stale);
    }
    Ok(DownloadRecoveryVerification::Verified(
        VerifiedDownloadRecovery {
            destination: snapshot.destination,
            repo_id: snapshot.repo_id,
            files: snapshot.selected_artifact_files,
        },
    ))
}

fn recovery_snapshot(
    library_root: &Path,
    record: &ModelRecord,
) -> Result<Option<RecoverySnapshot>> {
    let metadata = record
        .metadata
        .as_object()
        .ok_or_else(invalid_recovery_metadata)?;
    let incomplete = metadata
        .get("download_incomplete")
        .and_then(Value::as_bool)
        .ok_or_else(invalid_recovery_metadata)?;
    if !incomplete {
        return Ok(None);
    }

    let repo_id = optional_text(metadata, "repo_id")?;
    let Some(repo_id) = repo_id else {
        return Ok(None);
    };
    validate_repo_id(&repo_id)?;

    let selected_artifact_id = optional_text(metadata, "selected_artifact_id")?;
    let selected_artifact_quant = optional_text(metadata, "selected_artifact_quant")?;
    let selected_files = optional_file_set(metadata, "selected_artifact_files")?;
    let expected_files = optional_file_set(metadata, "expected_files")?;
    let files = if selected_files.is_empty() {
        expected_files
    } else {
        selected_files
    };

    if files.is_empty() {
        return Ok(None);
    }

    let Some(authority) = RecoveryRoot::open(library_root)? else {
        return Ok(None);
    };
    let authority = Arc::new(authority);
    let Some(destination) = authority.destination_for(record) else {
        return Ok(None);
    };
    if destination.preflight(&files).is_err() {
        return Ok(None);
    }

    let Some(canonical_model_dir) = destination.display_path.to_str().map(str::to_string) else {
        return Ok(None);
    };
    Ok(Some(RecoverySnapshot {
        model_id: record.id.clone(),
        canonical_model_dir,
        root_identity: authority.root_identity,
        destination,
        repo_id,
        selected_artifact_id,
        selected_artifact_files: files,
        selected_artifact_quant,
    }))
}

pub(crate) fn canonical_managed_model_dir(
    library_root: &Path,
    record: &ModelRecord,
) -> Result<Option<PathBuf>> {
    if !is_portable_relative_path(&record.id) {
        return Ok(None);
    }
    let canonical_root = std::fs::canonicalize(library_root).map_err(PumasError::from)?;
    let indexed_path = Path::new(&record.path);
    let indexed_metadata = match std::fs::symlink_metadata(indexed_path) {
        Ok(metadata) => metadata,
        Err(_) => return Ok(None),
    };
    if !indexed_metadata.is_dir() || indexed_metadata.file_type().is_symlink() {
        return Ok(None);
    }
    let canonical_model_dir = match std::fs::canonicalize(indexed_path) {
        Ok(path) => path,
        Err(_) => return Ok(None),
    };
    let Ok(relative) = canonical_model_dir.strip_prefix(&canonical_root) else {
        return Ok(None);
    };
    let relative_id = relative
        .components()
        .map(|component| match component {
            Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()
        .map(|components| components.join("/"));
    if relative_id.as_deref() != Some(record.id.as_str()) {
        return Ok(None);
    }
    Ok(Some(canonical_model_dir))
}

fn optional_text(metadata: &Map<String, Value>, field: &str) -> Result<Option<String>> {
    match metadata.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => {
            let value = value.trim();
            if value.is_empty() || value.len() > MAX_IDENTIFIER_BYTES {
                Err(invalid_recovery_metadata())
            } else {
                Ok(Some(value.to_string()))
            }
        }
        Some(_) => Err(invalid_recovery_metadata()),
    }
}

fn optional_file_set(metadata: &Map<String, Value>, field: &str) -> Result<Vec<String>> {
    let values = match metadata.get(field) {
        None | Some(Value::Null) => return Ok(Vec::new()),
        Some(Value::Array(values)) if values.len() <= MAX_COLLECTION_ITEMS => values,
        Some(Value::Array(_)) | Some(_) => return Err(invalid_recovery_metadata()),
    };
    let mut files = BTreeSet::new();
    for value in values {
        let value = value.as_str().ok_or_else(invalid_recovery_metadata)?;
        if !is_portable_relative_path(value) {
            return Err(invalid_recovery_metadata());
        }
        files.insert(value.to_string());
    }
    Ok(files.into_iter().collect())
}

fn validate_repo_id(value: &str) -> Result<()> {
    let mut segments = value.split('/');
    let owner = segments.next().unwrap_or_default();
    let name = segments.next().unwrap_or_default();
    let exact_shape = !owner.is_empty() && !name.is_empty() && segments.next().is_none();
    let valid_components = [owner, name].into_iter().all(|segment| {
        segment.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        }) && segment
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_alphanumeric() || character == '_')
            && segment
                .chars()
                .last()
                .is_some_and(|character| character.is_ascii_alphanumeric() || character == '_')
    });
    let forbidden = value.contains("--")
        || value.contains("..")
        || value.to_ascii_lowercase().ends_with(".git");
    if exact_shape && value.len() <= MAX_HF_REPO_ID_BYTES && valid_components && !forbidden {
        Ok(())
    } else {
        Err(invalid_recovery_metadata())
    }
}

fn is_portable_relative_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_IDENTIFIER_BYTES
        && !value.chars().any(|character| {
            character.is_control()
                || matches!(character, '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
        })
        && value.split('/').all(is_portable_path_component)
}

fn is_portable_path_component(component: &str) -> bool {
    if component.is_empty()
        || component.len() > MAX_PORTABLE_PATH_COMPONENT_BYTES
        || matches!(component, "." | "..")
        || component.ends_with(['.', ' '])
    {
        return false;
    }
    let stem = component.split('.').next().unwrap_or_default();
    let upper = stem.to_ascii_uppercase();
    !matches!(
        upper.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
    ) && !numbered_windows_device(&upper, "COM")
        && !numbered_windows_device(&upper, "LPT")
}

fn numbered_windows_device(value: &str, prefix: &str) -> bool {
    value
        .strip_prefix(prefix)
        .is_some_and(|suffix| matches!(suffix, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"))
}

fn invalid_recovery_metadata() -> PumasError {
    PumasError::Other("Model download recovery metadata is invalid".to_string())
}

fn invalid_capability_path() -> io::Error {
    io::Error::new(
        io::ErrorKind::PermissionDenied,
        "download recovery path is outside its verified filesystem authority",
    )
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    #[test]
    fn destination_identity_survives_aliases_missing_tail_and_creation() {
        let temp = tempfile::TempDir::new().unwrap();
        let root_path = temp.path().join("library");
        std::fs::create_dir(&root_path).unwrap();
        let alias = temp.path().join("alias");
        std::os::unix::fs::symlink(&root_path, &alias).unwrap();
        let root = super::DownloadDestinationRoot::open(&root_path).unwrap();
        let destination = root.resolve(std::path::Path::new("llm/model")).unwrap();
        let identity = destination.identity();
        assert_eq!(
            identity,
            root.resolve(&alias.join("llm/model")).unwrap().identity()
        );
        destination.prepare().unwrap();
        assert_eq!(
            identity,
            root.resolve(&root_path.join("llm/model"))
                .unwrap()
                .identity()
        );
        assert_eq!(identity.persisted().relative_target, "llm/model");
        let marker = serde_json::json!({"repo_id":"author/model", "files":["model.gguf"]});
        assert!(matches!(
            destination.write_marker(&marker).unwrap(),
            crate::metadata::AtomicPublication::Durable
        ));
        let actual: serde_json::Value = serde_json::from_slice(
            &std::fs::read(root_path.join("llm/model/.pumas_download")).unwrap(),
        )
        .unwrap();
        assert_eq!(actual, marker);
    }

    #[cfg(unix)]
    #[test]
    fn destination_rejects_nested_symlinks_and_escape() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = super::DownloadDestinationRoot::open(temp.path()).unwrap();
        std::fs::create_dir(temp.path().join("real")).unwrap();
        std::os::unix::fs::symlink("real", temp.path().join("alias")).unwrap();
        assert!(root.resolve(std::path::Path::new("alias/model")).is_err());
        assert!(root.resolve(std::path::Path::new("../escape")).is_err());
        assert!(root.resolve(std::path::Path::new(".")).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn destination_rejects_replaced_model_and_missing_target_parent() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = super::DownloadDestinationRoot::open(temp.path()).unwrap();
        std::fs::create_dir(temp.path().join("parent")).unwrap();
        let missing = root
            .resolve(std::path::Path::new("parent/missing"))
            .unwrap();
        std::fs::rename(temp.path().join("parent"), temp.path().join("old-parent")).unwrap();
        std::fs::create_dir(temp.path().join("parent")).unwrap();
        assert!(missing.prepare().is_err());
        assert!(!temp.path().join("parent/missing").exists());
        let destination = root.resolve(std::path::Path::new("model")).unwrap();
        destination.prepare().unwrap();
        std::fs::rename(temp.path().join("model"), temp.path().join("old-model")).unwrap();
        std::fs::create_dir(temp.path().join("model")).unwrap();
        assert!(destination.open_part("weights.gguf", false).is_err());
        assert!(destination
            .write_marker(&serde_json::json!({"files":[]}))
            .is_err());
        assert!(!temp.path().join("model/.pumas_download").exists());
    }

    #[cfg(unix)]
    #[test]
    fn destination_root_replacement_refuses_all_new_effects() {
        let temp = tempfile::TempDir::new().unwrap();
        let path = temp.path().join("root");
        std::fs::create_dir(&path).unwrap();
        let root = super::DownloadDestinationRoot::open(&path).unwrap();
        let destination = root.resolve(std::path::Path::new("missing")).unwrap();
        std::fs::rename(&path, temp.path().join("old-root")).unwrap();
        std::fs::create_dir(&path).unwrap();
        assert!(destination.prepare().is_err());
        assert!(root.resolve(std::path::Path::new("missing")).is_err());
        assert!(!path.join("missing").exists());
    }

    #[cfg(unix)]
    #[test]
    fn file_probes_distinguish_uncreated_paths_from_lost_authority() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = super::DownloadDestinationRoot::open(temp.path()).unwrap();
        let destination = root.resolve(std::path::Path::new("model")).unwrap();
        assert_eq!(destination.file_len("weights.gguf").unwrap(), None);
        assert_eq!(destination.part_len("weights.gguf").unwrap(), None);

        destination.prepare().unwrap();
        assert_eq!(destination.file_len("nested/weights.gguf").unwrap(), None);
        assert_eq!(destination.part_len("nested/weights.gguf").unwrap(), None);
        destination.create_parent("nested/weights.gguf").unwrap();
        std::fs::rename(
            temp.path().join("model/nested"),
            temp.path().join("model/old-nested"),
        )
        .unwrap();
        assert!(destination.file_len("nested/weights.gguf").is_err());
        assert!(destination.part_len("nested/weights.gguf").is_err());

        std::fs::rename(temp.path().join("model"), temp.path().join("old-model")).unwrap();
        assert!(destination.file_len("weights.gguf").is_err());
        assert!(destination.part_len("weights.gguf").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn cleanup_accepts_uncreated_target_but_refuses_lost_authority() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = super::DownloadDestinationRoot::open(temp.path()).unwrap();
        let destination = root.resolve(std::path::Path::new("model")).unwrap();
        destination.remove_part("weights.gguf").unwrap();
        destination.remove_marker().unwrap();
        assert!(destination.remove_part("../escape").is_err());
        assert!(!temp.path().join("model").exists());

        destination.prepare().unwrap();
        std::fs::remove_dir(temp.path().join("model")).unwrap();
        assert!(destination.remove_part("weights.gguf").is_err());
        assert!(destination.remove_marker().is_err());

        std::fs::create_dir(temp.path().join("parent")).unwrap();
        let anchored = root.resolve(std::path::Path::new("parent/model")).unwrap();
        std::fs::rename(temp.path().join("parent"), temp.path().join("old-parent")).unwrap();
        std::fs::create_dir(temp.path().join("parent")).unwrap();
        assert!(anchored.remove_part("weights.gguf").is_err());
        assert!(anchored.remove_marker().is_err());
    }

    #[cfg(unix)]
    #[test]
    fn cleanup_distinguishes_uncreated_nested_parent_from_lost_parent() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = super::DownloadDestinationRoot::open(temp.path()).unwrap();
        let destination = root.resolve(std::path::Path::new("model")).unwrap();
        destination.prepare().unwrap();
        destination.remove_part("nested/weights.gguf").unwrap();
        assert!(!temp.path().join("model/nested").exists());

        destination.create_parent("nested/weights.gguf").unwrap();
        std::fs::remove_dir(temp.path().join("model/nested")).unwrap();
        assert!(destination.remove_part("nested/weights.gguf").is_err());
        assert!(!temp.path().join("model/nested").exists());
    }

    #[cfg(unix)]
    #[test]
    fn ordinary_and_recovery_destinations_have_one_identity() {
        let temp = tempfile::TempDir::new().unwrap();
        let path = temp.path().join("library");
        let record = partial_record(&path, vec!["weights.gguf"]);
        let root = super::DownloadDestinationRoot::open(&path).unwrap();
        let ordinary = root.resolve(std::path::Path::new(&record.path)).unwrap();
        let recovery = std::sync::Arc::new(super::RecoveryRoot::open(&path).unwrap().unwrap())
            .destination_for(&record)
            .unwrap();
        assert_eq!(ordinary.identity(), recovery.identity());
        assert_eq!(
            ordinary.identity().persisted(),
            recovery.identity().persisted()
        );
    }

    #[cfg(unix)]
    #[test]
    fn destination_rejects_nested_parent_replacement_between_file_effects() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = super::DownloadDestinationRoot::open(temp.path()).unwrap();
        let destination = root.resolve(std::path::Path::new("model")).unwrap();
        destination.prepare().unwrap();
        let _part = destination.open_part("nested/model.bin", false).unwrap();
        let nested = temp.path().join("model/nested");
        std::fs::rename(&nested, temp.path().join("model/old-nested")).unwrap();
        std::fs::create_dir(&nested).unwrap();
        let replacement = nested.join("model.bin.part");
        std::fs::write(&replacement, b"replacement").unwrap();
        assert!(destination.rename_part_to_file("nested/model.bin").is_err());
        assert!(destination.remove_part("nested/model.bin").is_err());
        assert_eq!(std::fs::read(replacement).unwrap(), b"replacement");
        assert!(!nested.join("model.bin").exists());
    }
    use crate::ModelRecord;
    use serde_json::{json, Value};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tempfile::TempDir;

    use super::{
        issue_download_recovery_ticket, verify_download_recovery_ticket, DownloadRecoveryModelId,
        DownloadRecoveryToken, DownloadRecoveryVerification, RecoveryRoot,
    };

    fn partial_record(root: &std::path::Path, files: Vec<&str>) -> ModelRecord {
        let model_dir = root.join("llm/acme/model");
        std::fs::create_dir_all(&model_dir).unwrap();
        ModelRecord {
            id: "llm/acme/model".to_string(),
            path: model_dir.display().to_string(),
            cleaned_name: "model".to_string(),
            official_name: "Model".to_string(),
            model_type: "llm".to_string(),
            tags: Vec::new(),
            hashes: HashMap::new(),
            metadata: json!({
                "download_incomplete": true,
                "repo_id": "acme/model",
                "selected_artifact_id": "acme/model::Q4_K_M",
                "selected_artifact_files": files,
                "selected_artifact_quant": "Q4_K_M"
            }),
            updated_at: "2026-09-03T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn recovery_fingerprint_is_canonical_and_stales_on_semantic_change() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let original = partial_record(root, vec!["weights-2.gguf", "weights-1.gguf"]);
        let ticket = issue_download_recovery_ticket(root, &original)
            .unwrap()
            .unwrap();

        assert_eq!(ticket.token().len(), 67);
        assert!(ticket.token().starts_with("v1:"));
        assert!(ticket.token()[3..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit()));
        assert_eq!(
            ticket.selected_artifact_files(),
            &["weights-1.gguf".to_string(), "weights-2.gguf".to_string()]
        );

        let reordered = partial_record(root, vec!["weights-1.gguf", "weights-2.gguf"]);
        assert_eq!(
            issue_download_recovery_ticket(root, &reordered)
                .unwrap()
                .unwrap()
                .token(),
            ticket.token(),
            "file order is not semantic"
        );
        let duplicated = partial_record(
            root,
            vec!["weights-2.gguf", "weights-1.gguf", "weights-1.gguf"],
        );
        assert_eq!(
            issue_download_recovery_ticket(root, &duplicated)
                .unwrap()
                .unwrap()
                .token(),
            ticket.token(),
            "duplicate metadata entries normalize to the same set"
        );

        for changed in [
            partial_record(root, vec!["weights-1.gguf"]),
            partial_record(root, vec!["weights-1.gguf", "weights-3.gguf"]),
        ] {
            assert_ne!(
                issue_download_recovery_ticket(root, &changed)
                    .unwrap()
                    .unwrap()
                    .token(),
                ticket.token()
            );
        }

        for (field, value) in [
            ("repo_id", json!("Acme/model")),
            ("repo_id", json!("acme/other")),
            ("selected_artifact_id", json!("acme/model::Q5_K_M")),
            ("selected_artifact_quant", json!("Q5_K_M")),
        ] {
            let mut changed = original.clone();
            changed.metadata[field] = value;
            assert_ne!(
                issue_download_recovery_ticket(root, &changed)
                    .unwrap()
                    .unwrap()
                    .token(),
                ticket.token()
            );
        }

        let moved_dir = root.join("llm/acme/moved");
        std::fs::create_dir_all(&moved_dir).unwrap();
        let mut moved = original.clone();
        moved.id = "llm/acme/moved".to_string();
        moved.path = moved_dir.display().to_string();
        assert_ne!(
            issue_download_recovery_ticket(root, &moved)
                .unwrap()
                .unwrap()
                .token(),
            ticket.token()
        );

        let mut complete = original;
        complete.metadata["download_incomplete"] = json!(false);
        assert!(issue_download_recovery_ticket(root, &complete)
            .unwrap()
            .is_none());
    }

    #[test]
    fn recovery_action_identifiers_have_exact_portable_grammars() {
        assert!(DownloadRecoveryModelId::parse("llm/acme/model").is_some());
        for invalid in [
            "",
            "/llm/acme/model",
            "../model",
            "llm\\acme\\model",
            "C:/models/model",
            "llm//model",
            "llm/CON/model",
        ] {
            assert!(
                DownloadRecoveryModelId::parse(invalid).is_none(),
                "{invalid}"
            );
        }

        let valid = format!("v1:{}", "a".repeat(64));
        assert!(DownloadRecoveryToken::parse(&valid).is_some());
        for invalid in [
            "",
            "v1:abc",
            &format!("v2:{}", "a".repeat(64)),
            &format!("v1:{}", "A".repeat(64)),
            &format!("v1:{}", "g".repeat(64)),
            &format!("v1:{}", "a".repeat(65)),
        ] {
            assert!(DownloadRecoveryToken::parse(invalid).is_none(), "{invalid}");
        }
    }

    #[test]
    fn recovery_ticket_requires_managed_owned_path_and_provenance() {
        let temp = TempDir::new().unwrap();
        let root = temp.path().join("library");
        std::fs::create_dir_all(&root).unwrap();
        let managed = partial_record(&root, vec!["weights.gguf"]);
        assert!(issue_download_recovery_ticket(&root, &managed)
            .unwrap()
            .is_some());

        let outside_root = temp.path().join("outside/llm/acme/model");
        std::fs::create_dir_all(&outside_root).unwrap();
        let mut outside = managed.clone();
        outside.path = outside_root.display().to_string();
        assert!(issue_download_recovery_ticket(&root, &outside)
            .unwrap()
            .is_none());

        let mut alias = managed.clone();
        alias.id = "llm/acme/alias".to_string();
        assert!(issue_download_recovery_ticket(&root, &alias)
            .unwrap()
            .is_none());

        let mut missing = managed.clone();
        missing.path = root.join("llm/acme/missing").display().to_string();
        assert!(issue_download_recovery_ticket(&root, &missing)
            .unwrap()
            .is_none());

        let mut no_provenance = managed.clone();
        no_provenance.metadata["repo_id"] = Value::Null;
        assert!(issue_download_recovery_ticket(&root, &no_provenance)
            .unwrap()
            .is_none());

        let ticket = issue_download_recovery_ticket(&root, &managed)
            .unwrap()
            .unwrap();
        let stale = DownloadRecoveryToken::parse(&format!("v1:{}", "b".repeat(64))).unwrap();
        assert!(matches!(
            verify_download_recovery_ticket(&root, &managed, &stale).unwrap(),
            DownloadRecoveryVerification::Stale
        ));
        let current = DownloadRecoveryToken::parse(ticket.token()).unwrap();
        assert!(matches!(
            verify_download_recovery_ticket(&root, &managed, &current).unwrap(),
            DownloadRecoveryVerification::Verified(_)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn recovery_ticket_rejects_model_directory_symlinks() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new().unwrap();
        let root = temp.path().join("library");
        let target = root.join("llm/acme/target");
        let link = root.join("llm/acme/model");
        std::fs::create_dir_all(&target).unwrap();
        symlink(&target, &link).unwrap();
        let record = partial_record(&root, vec!["weights.gguf"]);
        assert!(issue_download_recovery_ticket(&root, &record)
            .unwrap()
            .is_none());
    }

    #[cfg(unix)]
    #[test]
    fn recovery_ticket_omits_authority_for_nested_and_partial_file_symlinks() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new().unwrap();
        let root = temp.path().join("library");
        let outside = temp.path().join("outside");
        std::fs::create_dir_all(&outside).unwrap();

        let nested = partial_record(&root, vec!["nested/weights.gguf"]);
        symlink(&outside, root.join("llm/acme/model/nested")).unwrap();
        assert!(issue_download_recovery_ticket(&root, &nested)
            .unwrap()
            .is_none());

        std::fs::remove_file(root.join("llm/acme/model/nested")).unwrap();
        let partial = partial_record(&root, vec!["weights.gguf"]);
        symlink(
            outside.join("escaped.part"),
            root.join("llm/acme/model/weights.gguf.part"),
        )
        .unwrap();
        assert!(issue_download_recovery_ticket(&root, &partial)
            .unwrap()
            .is_none());
        assert!(!outside.join("escaped.part").exists());
    }

    #[cfg(unix)]
    #[test]
    fn verified_authority_rejects_target_replacement_without_outside_mutation() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new().unwrap();
        let root = temp.path().join("library");
        let outside = temp.path().join("outside");
        std::fs::create_dir_all(&outside).unwrap();
        let record = partial_record(&root, vec!["weights.gguf"]);
        let ticket = issue_download_recovery_ticket(&root, &record)
            .unwrap()
            .unwrap();
        let token = DownloadRecoveryToken::parse(ticket.token()).unwrap();
        let DownloadRecoveryVerification::Verified(verified) =
            verify_download_recovery_ticket(&root, &record, &token).unwrap()
        else {
            panic!("recovery fixture must verify");
        };

        let original = root.join("llm/acme/model");
        let part = original.join("weights.gguf.part");
        symlink(outside.join("escaped.part"), &part).unwrap();
        assert!(verified
            .destination
            .open_part("weights.gguf", false)
            .is_err());
        assert!(!outside.join("escaped.part").exists());
        std::fs::remove_file(part).unwrap();

        std::fs::rename(&original, root.join("llm/acme/replaced-model")).unwrap();
        symlink(&outside, &original).unwrap();

        assert!(verified.destination.preflight(&verified.files).is_err());
        assert!(verified
            .destination
            .open_part("weights.gguf", false)
            .is_err());
        assert!(!outside.join("weights.gguf.part").exists());
    }

    #[test]
    fn recovery_token_rejects_library_root_replacement_with_same_display_path() {
        let temp = TempDir::new().unwrap();
        let root = temp.path().join("library");
        let record = partial_record(&root, vec!["weights.gguf"]);
        std::fs::write(root.join("old-root-sentinel"), b"old").unwrap();
        let ticket = issue_download_recovery_ticket(&root, &record)
            .unwrap()
            .unwrap();
        let token = DownloadRecoveryToken::parse(ticket.token()).unwrap();

        let original_root = temp.path().join("original-library");
        std::fs::rename(&root, &original_root).unwrap();
        let replacement = partial_record(&root, vec!["weights.gguf"]);
        std::fs::write(root.join("replacement-root-sentinel"), b"replacement").unwrap();

        assert!(matches!(
            verify_download_recovery_ticket(&root, &replacement, &token).unwrap(),
            DownloadRecoveryVerification::Stale | DownloadRecoveryVerification::Unavailable
        ));
        assert_eq!(
            std::fs::read(original_root.join("old-root-sentinel")).unwrap(),
            b"old"
        );
        assert_eq!(
            std::fs::read(root.join("replacement-root-sentinel")).unwrap(),
            b"replacement"
        );
    }

    #[test]
    fn held_root_rejects_replacement_before_model_validation() {
        let temp = TempDir::new().unwrap();
        let root = temp.path().join("library");
        let record = partial_record(&root, vec!["weights.gguf"]);
        std::fs::write(root.join("held-root-sentinel"), b"held").unwrap();
        let authority = Arc::new(RecoveryRoot::open(&root).unwrap().unwrap());

        let original_root = temp.path().join("original-library");
        std::fs::rename(&root, &original_root).unwrap();
        let replacement = partial_record(&root, vec!["weights.gguf"]);
        std::fs::write(root.join("replacement-root-sentinel"), b"replacement").unwrap();

        assert!(authority.destination_for(&replacement).is_none());
        assert_eq!(
            std::fs::read(original_root.join("held-root-sentinel")).unwrap(),
            b"held"
        );
        assert_eq!(record.path, replacement.path);
    }
}
