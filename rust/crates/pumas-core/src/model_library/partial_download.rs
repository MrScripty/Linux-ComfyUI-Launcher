//! Filesystem recovery for downloads interrupted after their final byte arrived.

use crate::error::{PumasError, Result};
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};

const DOWNLOAD_MARKER_FILENAME: &str = ".pumas_download";
const DOWNLOAD_TEMP_SUFFIX: &str = ".part";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct PartialArtifactFinalization {
    pub complete: bool,
    pub promoted_files: usize,
}

/// Filesystem operations used by the shared partial-artifact policy. The
/// adapter owns path authority and cleanup failure semantics.
pub(crate) trait PartialDownloadFiles {
    fn file_len(&self, filename: &str) -> Result<Option<u64>>;
    fn part_len(&self, filename: &str) -> Result<Option<u64>>;
    fn rename_part_to_file(&self, filename: &str) -> Result<()>;
    fn remove_part(&self, filename: &str) -> Result<()>;
    fn remove_marker(&self) -> Result<()>;
}

struct PathDownloadFiles<'a>(&'a Path);

impl PartialDownloadFiles for PathDownloadFiles<'_> {
    fn file_len(&self, filename: &str) -> Result<Option<u64>> {
        match download_artifact_paths(self.0, filename) {
            Some((path, _)) => regular_file_size(&path),
            None => Ok(None),
        }
    }

    fn part_len(&self, filename: &str) -> Result<Option<u64>> {
        match download_artifact_paths(self.0, filename) {
            Some((_, path)) => regular_file_size(&path),
            None => Ok(None),
        }
    }

    fn rename_part_to_file(&self, filename: &str) -> Result<()> {
        let (final_path, part_path) =
            download_artifact_paths(self.0, filename).ok_or_else(|| PumasError::InvalidParams {
                message: "Invalid partial artifact filename".into(),
            })?;
        std::fs::rename(&part_path, final_path)
            .map_err(|error| PumasError::io_with_path(error, part_path))
    }

    fn remove_part(&self, filename: &str) -> Result<()> {
        let (_, part_path) =
            download_artifact_paths(self.0, filename).ok_or_else(|| PumasError::InvalidParams {
                message: "Invalid partial artifact filename".into(),
            })?;
        if let Err(error) = std::fs::remove_file(&part_path) {
            if error.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!(
                    "Failed to remove stale completed download part {}: {}",
                    part_path.display(),
                    error
                );
            }
        }
        Ok(())
    }

    fn remove_marker(&self) -> Result<()> {
        let marker_path = self.0.join(DOWNLOAD_MARKER_FILENAME);
        if let Err(error) = std::fs::remove_file(&marker_path) {
            if error.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!(
                    "Failed to remove completed download marker {}: {}",
                    marker_path.display(),
                    error
                );
            }
        }
        Ok(())
    }
}

pub(crate) fn download_artifact_paths(
    model_dir: &Path,
    relative_path: &str,
) -> Option<(PathBuf, PathBuf)> {
    let relative = Path::new(relative_path);
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return None;
    }
    let final_path = model_dir.join(relative);
    let mut part_path = final_path.as_os_str().to_os_string();
    part_path.push(DOWNLOAD_TEMP_SUFFIX);
    Some((final_path, PathBuf::from(part_path)))
}

fn regular_file_size(path: &Path) -> Result<Option<u64>> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => Ok(Some(metadata.len())),
        Ok(_) => Ok(None),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(PumasError::io_with_path(error, path)),
    }
}

fn is_size_accounted_payload(relative_path: &str) -> bool {
    Path::new(relative_path)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "gguf" | "safetensors" | "bin" | "pt" | "pth" | "ckpt" | "onnx" | "npz"
            )
        })
}

/// Infer one unfinished file's expected size from the selected artifact total.
/// Multi-file inference is restricted to payload formats whose bytes are included
/// in Hugging Face's persisted LFS total; auxiliary files require remote sizes.
pub(crate) fn infer_expected_sizes_from_total(
    model_dir: &Path,
    expected_files: &[String],
    total_size: Option<u64>,
) -> Result<HashMap<String, u64>> {
    infer_expected_sizes_with_files(&PathDownloadFiles(model_dir), expected_files, total_size)
}

pub(crate) fn infer_expected_sizes_with_files(
    files: &impl PartialDownloadFiles,
    expected_files: &[String],
    total_size: Option<u64>,
) -> Result<HashMap<String, u64>> {
    let Some(total_size) = total_size.filter(|size| *size > 0) else {
        return Ok(HashMap::new());
    };
    if expected_files.len() == 1 {
        return Ok(HashMap::from([(expected_files[0].clone(), total_size)]));
    }
    if expected_files.is_empty()
        || !expected_files
            .iter()
            .all(|filename| is_size_accounted_payload(filename))
    {
        return Ok(HashMap::new());
    }

    let mut completed_size = 0_u64;
    let mut unfinished = Vec::new();
    let mut seen = HashSet::new();
    for relative_path in expected_files {
        if !seen.insert(relative_path) {
            continue;
        }
        if let Some(size) = files.file_len(relative_path)? {
            completed_size = completed_size.saturating_add(size);
        } else if files.part_len(relative_path)?.is_some() {
            unfinished.push(relative_path.clone());
        } else {
            return Ok(HashMap::new());
        }
    }

    if unfinished.len() != 1 {
        return Ok(HashMap::new());
    }
    let Some(expected_size) = total_size.checked_sub(completed_size) else {
        return Ok(HashMap::new());
    };
    if expected_size == 0 {
        return Ok(HashMap::new());
    }

    Ok(HashMap::from([(unfinished.remove(0), expected_size)]))
}

/// Promote every exact-size `.part` file once the selected artifact
/// is locally complete. Unknown-size partials are left untouched.
pub(crate) fn finalize_download_artifact_if_complete(
    model_dir: &Path,
    expected_files: &[String],
    expected_sizes: &HashMap<String, u64>,
) -> Result<PartialArtifactFinalization> {
    finalize_download_artifact_with_files(
        &PathDownloadFiles(model_dir),
        expected_files,
        expected_sizes,
    )
}

pub(crate) fn finalize_download_artifact_with_files(
    files: &impl PartialDownloadFiles,
    expected_files: &[String],
    expected_sizes: &HashMap<String, u64>,
) -> Result<PartialArtifactFinalization> {
    if expected_files.is_empty() {
        return Ok(PartialArtifactFinalization::default());
    }

    let mut seen = HashSet::new();
    let mut promotions = Vec::new();
    let mut stale_parts = Vec::new();

    for relative_path in expected_files {
        if !seen.insert(relative_path) {
            continue;
        }
        if files.file_len(relative_path)?.is_some() {
            if files.part_len(relative_path)?.is_some() {
                stale_parts.push(relative_path);
            }
            continue;
        }

        let Some(part_size) = files.part_len(relative_path)? else {
            return Ok(PartialArtifactFinalization::default());
        };
        let Some(expected_size) = expected_sizes.get(relative_path) else {
            return Ok(PartialArtifactFinalization::default());
        };
        if part_size != *expected_size {
            return Ok(PartialArtifactFinalization::default());
        }
        promotions.push(relative_path);
    }

    for filename in &promotions {
        files.rename_part_to_file(filename)?;
    }
    for filename in &stale_parts {
        files.remove_part(filename)?;
    }

    files.remove_marker()?;

    Ok(PartialArtifactFinalization {
        complete: true,
        promoted_files: promotions.len(),
    })
}
