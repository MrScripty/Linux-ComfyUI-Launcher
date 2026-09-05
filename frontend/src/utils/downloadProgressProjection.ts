import type { DownloadProgressOutcome } from '../generated/desktop-contract';
import type { ModelDownloadSnapshotEntry } from '../types/api-models';

/** RPC nullable absence maps to the existing pushed-snapshot presentation contract. */
export function projectDownloadProgress(progress: DownloadProgressOutcome): ModelDownloadSnapshotEntry {
  return {
    downloadId: progress.downloadId,
    status: progress.status,
    repoId: progress.repoId ?? undefined,
    selectedArtifactId: progress.selectedArtifactId,
    progress: progress.progress ?? undefined,
    downloadedBytes: progress.downloadedBytes ?? undefined,
    totalBytes: progress.totalBytes ?? undefined,
    speed: progress.speed ?? undefined,
    etaSeconds: progress.etaSeconds ?? undefined,
    modelName: progress.modelName ?? undefined,
    modelType: progress.modelType ?? undefined,
    retryAttempt: progress.retryAttempt ?? undefined,
    retryLimit: progress.retryLimit ?? undefined,
    retrying: progress.retrying ?? undefined,
    nextRetryDelaySeconds: progress.nextRetryDelaySeconds ?? undefined,
    error: progress.error ?? undefined,
  };
}
