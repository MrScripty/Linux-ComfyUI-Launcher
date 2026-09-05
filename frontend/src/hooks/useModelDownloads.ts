/**
 * Model Downloads Hook
 *
 * Manages model download state from backend snapshots and pushed updates.
 * Supports parallel downloads, pause/resume, and startup recovery.
 */

import { useState, useEffect, useRef, useCallback } from 'react';
import { api, getElectronAPI, isAPIAvailable } from '../api/adapter';
import type { ModelDownloadSnapshotEntry } from '../types/api';
import { getLogger } from '../utils/logger';
import { APIError } from '../errors';
import { projectDownloadProgress } from '../utils/downloadProgressProjection';
import {
  getDownloadArtifactKey,
  selectDownloadsByRepo,
  getUnusedDownloadKey,
  type DownloadStatus,
  type DownloadArtifactIdentity,
} from './modelDownloadState';

const logger = getLogger('useModelDownloads');

const ACTIVE_STATUSES = ['queued', 'downloading', 'cancelling', 'pausing'] as const;

function isActiveStatus(status: string): boolean {
  return (ACTIVE_STATUSES as readonly string[]).includes(status);
}

export function useModelDownloads() {
  const [downloadStatusByRepo, setDownloadStatusByRepo] = useState<Record<string, DownloadStatus>>({});
  const [downloadErrors, setDownloadErrors] = useState<Record<string, string>>({});

  // Ref keeps command handlers independent from render timing.
  const downloadStatusRef = useRef(downloadStatusByRepo);

  useEffect(() => {
    downloadStatusRef.current = downloadStatusByRepo;
  }, [downloadStatusByRepo]);

  const applyDownloadSnapshot = useCallback((
    downloads: ModelDownloadSnapshotEntry[],
    options: { preserveExisting?: boolean } = {}
  ) => {
    const { statuses, errors } = selectDownloadsByRepo(downloads);
    setDownloadStatusByRepo((prev) => {
      if (!options.preserveExisting) return statuses;
      const merged = { ...statuses };
      const snapshotKeys = new Map(Object.entries(statuses).map(([key, status]) => [status.downloadId, key]));
      for (const [key, status] of Object.entries(prev)) {
        const snapshotKey = snapshotKeys.get(status.downloadId);
        const targetKey = snapshotKey ?? (merged[key] && merged[key].downloadId !== status.downloadId
          ? getUnusedDownloadKey(status.downloadId, Object.keys(merged)) : key);
        merged[targetKey] = {
          ...statuses[targetKey], ...status,
          libraryModelId: snapshotKey ? statuses[snapshotKey]?.libraryModelId : status.libraryModelId,
        };
      }
      return merged;
    });
    setDownloadErrors(errors);
  }, []);

  // Startup recovery plus backend-owned pushed updates.
  useEffect(() => {
    let cancelled = false;

    const restoreDownloads = async () => {
      if (!isAPIAvailable()) return;
      try {
        const result = await api.list_model_downloads();
        if (!cancelled) {
          applyDownloadSnapshot(result.downloads.map(projectDownloadProgress), { preserveExisting: true });
        }
      } catch (error) {
        logger.warn('Failed to restore downloads on startup', { error });
      }
    };

    void restoreDownloads();

    const unsubscribe = getElectronAPI()?.onModelDownloadUpdate((notification) => {
      applyDownloadSnapshot(notification.snapshot.downloads.map(projectDownloadProgress));
    });

    return () => {
      cancelled = true;
      unsubscribe?.();
    };
  }, [applyDownloadSnapshot]);

  const startDownload = useCallback((
    downloadKey: string,
    downloadId: string,
    details?: { libraryModelId?: string | null; modelName?: string; modelType?: string } & DownloadArtifactIdentity
  ) => {
    const artifactKey = getDownloadArtifactKey({
      selectedArtifactId: details?.selectedArtifactId,
      artifactId: details?.artifactId,
      repoId: details?.repoId ?? downloadKey,
    }) ?? downloadKey;

    setDownloadStatusByRepo((prev) => {
      const existingEntry = Object.entries(prev).find(([, status]) => status.downloadId === downloadId);
      const targetKey = existingEntry?.[0] ?? (prev[artifactKey]
        ? getUnusedDownloadKey(downloadId, Object.keys(prev)) : artifactKey);
      const existing = prev[targetKey];
      if (existing && isActiveStatus(existing.status)) {
        if (existing.downloadId === downloadId && details?.libraryModelId != null
          && existing.libraryModelId !== details.libraryModelId) {
          return { ...prev, [targetKey]: { ...existing, libraryModelId: details.libraryModelId } };
        }
        return prev;
      }
      return {
        ...prev,
        [targetKey]: {
          downloadId,
          libraryModelId: details?.libraryModelId
            ?? (existing?.downloadId === downloadId ? existing.libraryModelId : undefined),
          status: 'queued',
          progress: 0,
          repoId: details?.repoId ?? downloadKey,
          selectedArtifactId: details?.selectedArtifactId,
          artifactId: details?.artifactId,
          modelName: details?.modelName,
          modelType: details?.modelType,
        },
      };
    });
    setDownloadErrors((prev) => {
      const keys = Object.keys(prev).filter((key) => downloadStatusRef.current[key]?.downloadId === downloadId
        || (key === artifactKey && !downloadStatusRef.current[key]));
      if (keys.length === 0) return prev;
      const next = { ...prev };
      for (const key of keys) delete next[key];
      return next;
    });
  }, []);

  const cancelDownload = useCallback(async (downloadKey: string) => {
    const status = downloadStatusRef.current[downloadKey];
    if (!status || !isAPIAvailable()) return;

    setDownloadStatusByRepo((prev) => ({
      ...prev,
      [downloadKey]: {
        ...prev[downloadKey],
        downloadId: prev[downloadKey]?.downloadId || status.downloadId,
        status: 'cancelling' as const,
        progress: prev[downloadKey]?.progress || 0,
      },
    }));

    try {
      await api.cancel_model_download(status.downloadId);
    } catch (error) {
      if (error instanceof APIError) {
        logger.error('API error cancelling download', { error: error.message, endpoint: error.endpoint, downloadKey, repoId: status.repoId });
      } else if (error instanceof Error) {
        logger.error('Unexpected error cancelling download', { error: error.message, downloadKey, repoId: status.repoId });
      } else {
        logger.error('Unknown error cancelling download', { error, downloadKey, repoId: status.repoId });
      }
    }
  }, []);

  const pauseDownload = useCallback(async (downloadKey: string) => {
    const status = downloadStatusRef.current[downloadKey];
    if (!status || !isAPIAvailable()) return;

    setDownloadStatusByRepo((prev) => {
      const existing = prev[downloadKey];
      if (!existing) return prev;
      return { ...prev, [downloadKey]: { ...existing, status: 'pausing' as const } };
    });

    try {
      await api.pause_model_download(status.downloadId);
    } catch (error) {
      logger.error('Failed to pause download', {
        error: error instanceof Error ? error.message : error,
        downloadKey,
        repoId: status.repoId,
      });
    }
  }, []);

  const resumeDownload = useCallback(async (downloadKey: string) => {
    const status = downloadStatusRef.current[downloadKey];
    if (!status || !isAPIAvailable()) return;

    setDownloadStatusByRepo((prev) => {
      const existing = prev[downloadKey];
      if (!existing) return prev;
      return { ...prev, [downloadKey]: { ...existing, status: 'queued' as const, speed: undefined, etaSeconds: undefined } };
    });
    setDownloadErrors((prev) => {
      if (!prev[downloadKey]) return prev;
      const next = { ...prev };
      delete next[downloadKey];
      return next;
    });

    try {
      const result = await api.resume_model_download(status.downloadId);
      if (!result.success) {
        throw new APIError(result.error || 'Failed to resume download.', 'resume_model_download');
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to resume download.';
      setDownloadStatusByRepo((prev) => {
        const existing = prev[downloadKey];
        if (!existing) return prev;
        return { ...prev, [downloadKey]: { ...existing, status: 'error' as const } };
      });
      setDownloadErrors((prev) => ({ ...prev, [downloadKey]: message }));
      logger.error('Failed to resume download', {
        error: message,
        downloadKey,
        repoId: status.repoId,
      });
    }
  }, []);

  const hasActiveDownloads = Object.values(downloadStatusByRepo).some((s) => isActiveStatus(s.status));

  return {
    downloadStatusByRepo,
    downloadErrors,
    hasActiveDownloads,
    startDownload,
    cancelDownload,
    pauseDownload,
    resumeDownload,
    setDownloadErrors,
  };
}
