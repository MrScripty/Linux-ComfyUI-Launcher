//! Migration report and metadata-v2 move methods on `PumasApi`.

use super::{reconcile_on_demand, ReconcileScope};
use crate::error::{PumasError, Result};
use crate::model_library;
use crate::models;
use crate::PumasApi;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

const MIGRATION_REPORTS_DIR: &str = "migration-reports";

fn normalize_absolute_local_path(value: &str, field: &str) -> Result<PathBuf> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(PumasError::InvalidParams {
            message: format!("{field} is required"),
        });
    }

    let raw = PathBuf::from(trimmed);
    let mut normalized = if raw.is_absolute() {
        PathBuf::new()
    } else {
        std::env::current_dir().map_err(|err| {
            PumasError::Other(format!(
                "Failed to resolve current directory for {field}: {}",
                err
            ))
        })?
    };

    for component in raw.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
        }
    }

    Ok(normalized)
}

pub(crate) fn normalize_migration_report_path(
    library_root: &Path,
    report_path: &str,
) -> Result<PathBuf> {
    let raw = report_path.trim();
    if raw.is_empty() {
        return Err(PumasError::InvalidParams {
            message: "report_path is required".to_string(),
        });
    }

    let normalized = if Path::new(raw).is_absolute() {
        normalize_absolute_local_path(raw, "report_path")?
    } else {
        let reports_root = library_root.join(MIGRATION_REPORTS_DIR);
        normalize_absolute_local_path(
            reports_root.join(raw).to_string_lossy().as_ref(),
            "report_path",
        )?
    };

    let reports_root = normalize_absolute_local_path(
        library_root
            .join(MIGRATION_REPORTS_DIR)
            .to_string_lossy()
            .as_ref(),
        "report_path",
    )?;
    if !normalized.starts_with(&reports_root) {
        return Err(PumasError::InvalidParams {
            message: format!(
                "report_path must be within migration reports directory: {}",
                normalized.display()
            ),
        });
    }

    Ok(normalized)
}

impl PumasApi {
    /// Generate a non-mutating migration dry-run report for metadata v2 cutover.
    pub async fn generate_model_migration_dry_run_report(
        &self,
    ) -> Result<model_library::MigrationDryRunReport> {
        let primary = self.primary();
        reconcile_all_models_for_migration(
            primary.as_ref(),
            "api-generate-model-migration-dry-run-report",
        )
        .await?;

        generate_migration_dry_run_report_with_artifacts(primary.model_library.clone()).await
    }

    /// Execute checkpointed metadata v2 migration moves.
    pub async fn execute_model_migration(&self) -> Result<model_library::MigrationExecutionReport> {
        let primary = self.primary();
        reconcile_all_models_for_migration(primary.as_ref(), "api-execute-model-migration").await?;

        let mut report = primary
            .model_library
            .execute_migration_with_checkpoint()
            .await?;
        let mutated = relocate_skipped_partial_downloads(
            &primary.model_library,
            primary.hf_client.as_ref(),
            &mut report,
        )
        .await?;
        if mutated {
            recompute_execution_report_counts(&mut report);
            // Rewrite artifacts so UI/opened report JSON reflects post-move outcomes.
            rewrite_migration_execution_report(primary.model_library.clone(), report.clone())
                .await?;
        }
        primary
            .model_library
            .notify_model_library_refresh("migration_execution")?;
        Ok(report)
    }

    /// List migration report artifacts from the report index (newest-first).
    pub async fn list_model_migration_reports(
        &self,
    ) -> Result<Vec<model_library::MigrationReportArtifact>> {
        list_migration_reports(self.primary().model_library.clone()).await
    }

    /// Delete a migration report artifact pair (JSON + Markdown) and index entry.
    pub async fn delete_model_migration_report(&self, report_path: &str) -> Result<bool> {
        let normalized = normalize_migration_report_path(
            self.primary().model_library.library_root(),
            report_path,
        )?;

        delete_migration_report(
            self.primary().model_library.clone(),
            normalized.to_string_lossy().to_string(),
        )
        .await
    }

    /// Prune migration report history to `keep_latest` entries.
    pub async fn prune_model_migration_reports(&self, keep_latest: usize) -> Result<usize> {
        prune_migration_reports(self.primary().model_library.clone(), keep_latest).await
    }
}

pub(crate) async fn generate_migration_dry_run_report_with_artifacts(
    library: Arc<model_library::ModelLibrary>,
) -> Result<model_library::MigrationDryRunReport> {
    tokio::task::spawn_blocking(move || library.generate_migration_dry_run_report_with_artifacts())
        .await
        .map_err(|err| {
            PumasError::Other(format!(
                "Failed to join migration dry-run report task: {}",
                err
            ))
        })?
}

pub(crate) async fn rewrite_migration_execution_report(
    library: Arc<model_library::ModelLibrary>,
    report: model_library::MigrationExecutionReport,
) -> Result<()> {
    tokio::task::spawn_blocking(move || library.rewrite_migration_execution_report(&report))
        .await
        .map_err(|err| {
            PumasError::Other(format!(
                "Failed to join migration execution report rewrite task: {}",
                err
            ))
        })?
}

pub(crate) async fn list_migration_reports(
    library: Arc<model_library::ModelLibrary>,
) -> Result<Vec<model_library::MigrationReportArtifact>> {
    tokio::task::spawn_blocking(move || library.list_migration_reports())
        .await
        .map_err(|err| {
            PumasError::Other(format!(
                "Failed to join migration report listing task: {}",
                err
            ))
        })?
}

pub(crate) async fn delete_migration_report(
    library: Arc<model_library::ModelLibrary>,
    report_path: String,
) -> Result<bool> {
    tokio::task::spawn_blocking(move || library.delete_migration_report(&report_path))
        .await
        .map_err(|err| {
            PumasError::Other(format!(
                "Failed to join migration report delete task: {}",
                err
            ))
        })?
}

pub(crate) async fn prune_migration_reports(
    library: Arc<model_library::ModelLibrary>,
    keep_latest: usize,
) -> Result<usize> {
    tokio::task::spawn_blocking(move || library.prune_migration_reports(keep_latest))
        .await
        .map_err(|err| {
            PumasError::Other(format!(
                "Failed to join migration report prune task: {}",
                err
            ))
        })?
}

async fn reconcile_all_models_for_migration(
    primary: &super::state::PrimaryState,
    reason: &'static str,
) -> Result<()> {
    primary.reconciliation.mark_dirty_all().await;
    let _ = reconcile_on_demand(primary, ReconcileScope::AllModels, reason).await?;
    Ok(())
}

pub(crate) fn split_model_id(model_id: &str) -> Option<(&str, &str, &str)> {
    let mut parts = model_id.splitn(3, '/');
    let model_type = parts.next()?;
    let family = parts.next()?;
    let cleaned_name = parts.next()?;
    Some((model_type, family, cleaned_name))
}

pub(crate) async fn wait_for_download_pause(
    client: &model_library::HuggingFaceClient,
    download_id: &str,
) -> Result<()> {
    for _ in 0..80 {
        match client.get_download_status(download_id).await {
            Some(models::DownloadStatus::Paused)
            | Some(models::DownloadStatus::Error)
            | Some(models::DownloadStatus::Cancelled)
            | Some(models::DownloadStatus::Completed) => return Ok(()),
            Some(models::DownloadStatus::Downloading)
            | Some(models::DownloadStatus::Queued)
            | Some(models::DownloadStatus::Pausing)
            | Some(models::DownloadStatus::Cancelling) => {
                sleep(Duration::from_millis(250)).await;
            }
            None => {
                return Err(PumasError::NotFound {
                    resource: format!("download_id {}", download_id),
                });
            }
        }
    }

    Err(PumasError::Other(format!(
        "Timed out waiting for download {} to pause before migration move",
        download_id
    )))
}

pub(crate) async fn relocate_skipped_partial_downloads(
    library: &model_library::ModelLibrary,
    hf_client: Option<&model_library::HuggingFaceClient>,
    report: &mut model_library::MigrationExecutionReport,
) -> Result<bool> {
    let mut mutated = false;
    for row in &mut report.results {
        if row.action != "skipped_partial_download" {
            continue;
        }
        let Some((target_model_type, target_family, target_cleaned_name)) =
            split_model_id(&row.target_model_id)
        else {
            row.action = "partial_move_error".into();
            row.error = Some(format!("Invalid target model_id: {}", row.target_model_id));
            mutated = true;
            continue;
        };
        let Some(client) = hf_client else {
            row.error =
                Some("Partial download retained: download lifecycle owner unavailable".into());
            mutated = true;
            continue;
        };
        let source_dir = library.library_root().join(&row.model_id);
        let target_dir =
            library.build_model_path(target_model_type, target_family, target_cleaned_name);
        let mut relocation_completed = false;
        let move_result: Result<bool> = async {
            let Some(download_id) = client.download_owner_for_move(&source_dir).await? else {
                return Ok(false);
            };
            let metadata = library
                .index()
                .get(&row.model_id)?
                .map(|record| {
                    serde_json::from_value::<model_library::ModelMetadata>(record.metadata)
                })
                .transpose()?;
            let status = client.get_download_status(&download_id).await;
            let resume_after_move = matches!(
                status,
                Some(
                    models::DownloadStatus::Queued
                        | models::DownloadStatus::Downloading
                        | models::DownloadStatus::Pausing
                )
            );
            if resume_after_move {
                if status != Some(models::DownloadStatus::Pausing)
                    && !client.pause_download(&download_id).await?
                {
                    return Err(PumasError::Validation {
                        field: "download_relocation".into(),
                        message: "Download owner refused migration pause".into(),
                    });
                }
                wait_for_download_pause(client, &download_id).await?;
            }
            if !client
                .relocate_download_destination_from(
                    &download_id,
                    &source_dir,
                    &target_dir,
                    Some(target_model_type),
                    Some(target_family),
                )
                .await?
            {
                return Err(PumasError::Validation {
                    field: "download_relocation".into(),
                    message: "Download owner refused relocation before movement".into(),
                });
            }
            relocation_completed = true;
            if let Some(mut metadata) = metadata {
                metadata.model_id = Some(row.target_model_id.clone());
                metadata.model_type = Some(target_model_type.to_string());
                metadata.family = Some(target_family.to_string());
                metadata.cleaned_name = Some(target_cleaned_name.to_string());
                metadata.updated_date = Some(chrono::Utc::now().to_rfc3339());
                library.upsert_index_from_metadata(&target_dir, &metadata)?;
                library.index().delete(&row.model_id)?;
            }
            // A later index or resume failure cannot authorize a physical rollback.
            if resume_after_move && !client.resume_download(&download_id).await? {
                return Err(PumasError::Validation {
                    field: "download_relocation".into(),
                    message: "Download moved, but its owner refused resumption".into(),
                });
            }
            Ok(true)
        }
        .await;
        match move_result {
            Ok(true) => {
                row.action = "moved_partial".into();
                row.error = None;
            }
            Ok(false) => {
                row.error = Some("Partial download retained: no tracked download owner".into());
            }
            Err(error) => {
                if relocation_completed {
                    row.action = "moved_partial".into();
                    row.error = Some(format!("Download moved; post-move update failed: {error}"));
                } else {
                    row.action = "partial_move_error".into();
                    row.error = Some(error.to_string());
                }
            }
        }
        mutated = true;
    }
    Ok(mutated)
}

pub(crate) fn recompute_execution_report_counts(
    report: &mut model_library::MigrationExecutionReport,
) {
    report.completed_move_count = 0;
    report.skipped_move_count = 0;
    report.error_count = 0;
    for row in &report.results {
        if row.action == "moved_partial" && row.error.is_some() {
            report.error_count += 1;
        }
        match row.action.as_str() {
            "moved" | "already_migrated" | "moved_partial" => report.completed_move_count += 1,
            "blocked_collision" | "missing_source" | "skipped_partial_download" => {
                report.skipped_move_count += 1
            }
            _ => report.error_count += 1,
        }
    }
    if !report.referential_integrity_ok {
        report.error_count += report.referential_integrity_errors.len();
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_migration_report_path;
    use tempfile::TempDir;

    #[tokio::test]
    async fn partial_migration_preserves_bytes_when_download_owner_refuses() {
        let temp = TempDir::new().unwrap();
        let library = crate::model_library::ModelLibrary::new(temp.path().join("models"))
            .await
            .unwrap();
        let source = library.library_root().join("llm/old/model");
        let target = library.library_root().join("llm/new/model");
        std::fs::create_dir_all(&source).unwrap();
        std::fs::write(source.join("weights.gguf.part"), b"partial bytes").unwrap();
        let mut client = crate::model_library::HuggingFaceClient::new(temp.path()).unwrap();
        client
            .configure_download_destination_root(library.library_root())
            .unwrap();
        let persistence =
            std::sync::Arc::new(crate::model_library::DownloadPersistence::new(temp.path()));
        let snapshot: crate::model_library::download_store::PersistedDownload =
            serde_json::from_value(serde_json::json!({
                "download_id": "not-restored",
                "repo_id": "owner/model",
                "filename": "weights.gguf",
                "filenames": ["weights.gguf"],
                "dest_dir": source,
                "total_bytes": 100,
                "status": "paused",
                "download_request": {
                    "repo_id": "owner/model", "family": "old", "official_name": "Model"
                },
                "created_at": "2026-09-04T00:00:00Z"
            }))
            .unwrap();
        persistence.save(&snapshot).unwrap();
        client.set_persistence(persistence.clone());
        // The persisted row exists, but no runtime owner has restored it.
        // A refusal must not be treated as a successful filesystem move.
        let mut report = crate::model_library::MigrationExecutionReport {
            results: vec![crate::model_library::MigrationExecutionItem {
                model_id: "llm/old/model".into(),
                target_model_id: "llm/new/model".into(),
                action: "skipped_partial_download".into(),
                error: None,
            }],
            ..Default::default()
        };
        super::relocate_skipped_partial_downloads(&library, Some(&client), &mut report)
            .await
            .unwrap();
        assert_eq!(report.results[0].action, "partial_move_error");
        assert!(report.results[0]
            .error
            .as_deref()
            .unwrap()
            .contains("refused"));
        assert_eq!(
            std::fs::read(source.join("weights.gguf.part")).unwrap(),
            b"partial bytes"
        );
        assert!(!target.exists());
        assert_eq!(persistence.load_all_strict().unwrap()[0].dest_dir, source);

        let marker = serde_json::json!({
            "repo_id": "owner/model", "family": "old", "model_type": "llm",
            "selected_artifact": {"artifact_id": "selected-q4", "selected_quant": "q4_k_m"}
        });
        std::fs::write(
            source.join(".pumas_download"),
            serde_json::to_vec(&marker).unwrap(),
        )
        .unwrap();
        library
            .upsert_index_from_metadata(
                &source,
                &crate::model_library::ModelMetadata {
                    model_id: Some("llm/old/model".into()),
                    family: Some("old".into()),
                    model_type: Some("llm".into()),
                    official_name: Some("Model".into()),
                    cleaned_name: Some("model".into()),
                    ..Default::default()
                },
            )
            .unwrap();
        client.restore_persisted_downloads().await.unwrap();
        report.results[0].action = "skipped_partial_download".into();
        super::relocate_skipped_partial_downloads(&library, Some(&client), &mut report)
            .await
            .unwrap();
        assert_eq!(
            report.results[0].action, "moved_partial",
            "{:?}",
            report.results[0].error
        );
        assert!(report.results[0].error.is_none());
        assert!(!source.exists());
        assert_eq!(
            std::fs::read(target.join("weights.gguf.part")).unwrap(),
            b"partial bytes"
        );
        let actual: serde_json::Value =
            serde_json::from_slice(&std::fs::read(target.join(".pumas_download")).unwrap())
                .unwrap();
        assert_eq!(actual["family"], "new");
        assert_eq!(actual["architecture_family"], "new");
        assert_eq!(actual["selected_artifact"], marker["selected_artifact"]);
        let reopened = crate::model_library::DownloadPersistence::new(temp.path());
        assert_eq!(reopened.load_all_strict().unwrap()[0].dest_dir, target);
        assert!(library.index().get("llm/old/model").unwrap().is_none());
        assert!(library.index().get("llm/new/model").unwrap().is_some());
    }

    #[test]
    fn normalize_migration_report_path_accepts_relative_report_path() {
        let temp_dir = TempDir::new().expect("temp dir");
        let normalized =
            normalize_migration_report_path(temp_dir.path(), "dry-run/report-20260425.md")
                .expect("relative report path should normalize");

        assert_eq!(
            normalized,
            temp_dir
                .path()
                .join("migration-reports")
                .join("dry-run")
                .join("report-20260425.md")
        );
    }

    #[test]
    fn normalize_migration_report_path_rejects_path_outside_reports_root() {
        let temp_dir = TempDir::new().expect("temp dir");
        let outside = temp_dir.path().join("outside.md");

        let error =
            normalize_migration_report_path(temp_dir.path(), outside.to_string_lossy().as_ref())
                .expect_err("path outside migration reports root should be rejected");

        assert!(matches!(
            error,
            crate::error::PumasError::InvalidParams { message }
                if message.contains("within migration reports directory")
        ));
    }
}
