import { act, renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const {
  cancelModelDownloadMock,
  getElectronAPIMock,
  isApiAvailableMock,
  listModelDownloadsMock,
  pauseModelDownloadMock,
  resumeModelDownloadMock,
} = vi.hoisted(() => ({
  cancelModelDownloadMock: vi.fn(),
  getElectronAPIMock: vi.fn(),
  isApiAvailableMock: vi.fn<() => boolean>(),
  listModelDownloadsMock: vi.fn(),
  pauseModelDownloadMock: vi.fn(),
  resumeModelDownloadMock: vi.fn(),
}));

vi.mock('../api/adapter', () => ({
  api: {
    cancel_model_download: cancelModelDownloadMock,
    list_model_downloads: listModelDownloadsMock,
    pause_model_download: pauseModelDownloadMock,
    resume_model_download: resumeModelDownloadMock,
  },
  getElectronAPI: getElectronAPIMock,
  isAPIAvailable: isApiAvailableMock,
}));

import type { ModelDownloadUpdateNotification } from '../types/api';
import type { DownloadProgressOutcome } from '../generated/desktop-contract';
import { useModelDownloads } from './useModelDownloads';
import { buildDownloadingModels, mergeLocalModelGroups } from '../components/ModelManagerUtils';

function progressOutcome(overrides: Partial<DownloadProgressOutcome>): DownloadProgressOutcome {
  return {
    downloadId: 'dl-1', status: 'downloading', repoId: 'org/model', selectedArtifactId: 'artifact-1',
    libraryModelId: null, progress: 40, downloadedBytes: 4, totalBytes: 10, speed: null,
    etaSeconds: null, modelName: null, modelType: null, retryAttempt: null, retryLimit: null,
    retrying: null, nextRetryDelaySeconds: null, error: null, ...overrides,
  };
}

async function flushMicrotasks() {
  await act(async () => {
    await Promise.resolve();
  });
}

describe('useModelDownloads', () => {
  let downloadUpdateCallback: ((notification: ModelDownloadUpdateNotification) => void) | null;
  let unsubscribeMock: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
    downloadUpdateCallback = null;
    unsubscribeMock = vi.fn();
    isApiAvailableMock.mockReturnValue(true);
    getElectronAPIMock.mockReturnValue({
      onModelDownloadUpdate: vi.fn((callback: (notification: ModelDownloadUpdateNotification) => void) => {
        downloadUpdateCallback = callback;
        return unsubscribeMock;
      }),
    });
    listModelDownloadsMock.mockResolvedValue({
      success: true,
      downloads: [],
    });
    cancelModelDownloadMock.mockResolvedValue({ success: true });
    pauseModelDownloadMock.mockResolvedValue({ success: true });
    resumeModelDownloadMock.mockResolvedValue({ success: true });
  });

  afterEach(() => {
    vi.clearAllTimers();
    vi.useRealTimers();
  });

  it('propagates authoritative catalog association from startup and canonical pushes', async () => {
    listModelDownloadsMock.mockResolvedValueOnce({ success: true, downloads: [
      progressOutcome({ libraryModelId: 'llm/org/model' }),
    ] });
    const { result } = renderHook(() => useModelDownloads());
    await flushMicrotasks();
    expect(result.current.downloadStatusByRepo['artifact-1']?.libraryModelId).toBe('llm/org/model');
    act(() => downloadUpdateCallback?.({
      cursor: 'download:2', snapshot: { cursor: 'download:2', revision: 2, downloads: [
        progressOutcome({ libraryModelId: 'llm/org/model', downloadedBytes: null, progress: 55 }),
      ] }, stale_cursor: false, snapshot_required: false,
    }));
    expect(result.current.downloadStatusByRepo['artifact-1']).toMatchObject({
      libraryModelId: 'llm/org/model', progress: 55, downloadedBytes: undefined,
    });
  });

  it('associates recovery immediately and reuses an exact pushed download ID on late start acknowledgement', async () => {
    const { result } = renderHook(() => useModelDownloads());
    await flushMicrotasks();
    act(() => result.current.startDownload('recovery-request', 'dl-recovery', {
      libraryModelId: 'llm/org/recovery', repoId: 'org/recovery',
    }));
    expect(result.current.downloadStatusByRepo['org/recovery']?.libraryModelId).toBe('llm/org/recovery');
    act(() => downloadUpdateCallback?.({
      cursor: 'download:3', snapshot: { cursor: 'download:3', revision: 3, downloads: [
        progressOutcome({ downloadId: 'dl-recovery', libraryModelId: 'llm/org/recovery' }),
      ] }, stale_cursor: false, snapshot_required: false,
    }));
    act(() => result.current.startDownload('late-ack-key', 'dl-recovery', {
      libraryModelId: 'llm/org/recovery', repoId: 'org/recovery',
    }));
    expect(Object.keys(result.current.downloadStatusByRepo)).toEqual(['artifact-1']);
    expect(result.current.downloadStatusByRepo['artifact-1']).toMatchObject({
      downloadId: 'dl-recovery', libraryModelId: 'llm/org/recovery', status: 'downloading', progress: 40,
    });
    await act(async () => { await result.current.pauseDownload('artifact-1'); });
    expect(pauseModelDownloadMock).toHaveBeenCalledWith('dl-recovery');
    act(() => downloadUpdateCallback?.({
      cursor: 'download:4', snapshot: { cursor: 'download:4', revision: 4, downloads: [
        progressOutcome({ downloadId: 'dl-recovery', libraryModelId: 'llm/org/recovery', status: 'error', error: 'Disk full' }),
      ] }, stale_cursor: false, snapshot_required: false,
    }));
    act(() => result.current.startDownload('late-ack-key', 'dl-recovery', { libraryModelId: 'llm/org/recovery' }));
    expect(Object.keys(result.current.downloadStatusByRepo)).toEqual(['artifact-1']);
    expect(result.current.downloadStatusByRepo['artifact-1']?.status).toBe('queued');
    expect(result.current.downloadErrors).toEqual({});
  });

  it('merges a late startup snapshot by exact download ID without retaining an optimistic alias', async () => {
    let resolveList!: (value: { success: true; downloads: DownloadProgressOutcome[] }) => void;
    listModelDownloadsMock.mockReturnValueOnce(new Promise((resolve) => { resolveList = resolve; }));
    const { result } = renderHook(() => useModelDownloads());
    act(() => result.current.startDownload('org/model', 'dl-1', { libraryModelId: 'llm/org/model' }));
    await act(async () => resolveList({ success: true, downloads: [
      progressOutcome({ libraryModelId: 'llm/org/model' }),
    ] }));
    expect(Object.keys(result.current.downloadStatusByRepo)).toEqual(['artifact-1']);
    expect(result.current.downloadStatusByRepo['artifact-1']).toMatchObject({
      downloadId: 'dl-1', libraryModelId: 'llm/org/model',
    });
  });

  it('does not inherit catalog association when a different download reuses an inactive artifact key', async () => {
    listModelDownloadsMock.mockResolvedValueOnce({ success: true, downloads: [
      progressOutcome({ downloadId: 'dl-old', status: 'error', libraryModelId: 'llm/org/old' }),
    ] });
    const { result } = renderHook(() => useModelDownloads());
    await flushMicrotasks();
    act(() => result.current.startDownload('artifact-1', 'dl-new', { selectedArtifactId: 'artifact-1' }));
    expect(Object.values(result.current.downloadStatusByRepo).find(status => status.downloadId === 'dl-new')).toMatchObject({
      downloadId: 'dl-new', libraryModelId: undefined, status: 'queued',
    });
  });

  it('retains distinct optimistic IDs when an initial list arrives with the same artifact', async () => {
    let resolveList: ((value: { success: boolean; downloads: DownloadProgressOutcome[] }) => void) | undefined;
    listModelDownloadsMock.mockReturnValueOnce(new Promise(resolve => { resolveList = resolve; }));
    const { result } = renderHook(() => useModelDownloads());
    act(() => {
      result.current.startDownload('artifact-1', 'dl-local', { selectedArtifactId: 'artifact-1', libraryModelId: 'local' });
      result.current.startDownload('artifact-1', 'dl-other', { selectedArtifactId: 'artifact-1', libraryModelId: 'other' });
    });
    expect(Object.values(result.current.downloadStatusByRepo).map(status => status.downloadId).sort())
      .toEqual(['dl-local', 'dl-other']);
    await act(async () => { resolveList?.({ success: true, downloads: [
      progressOutcome({ downloadId: 'dl-snapshot', libraryModelId: 'snapshot' }),
    ] }); });
    expect(Object.values(result.current.downloadStatusByRepo).map(status => [status.downloadId, status.libraryModelId]).sort())
      .toEqual([['dl-local', 'local'], ['dl-other', 'other'], ['dl-snapshot', 'snapshot']]);
    expect(result.current.downloadStatusByRepo['artifact-1']?.downloadId).toBe('dl-snapshot');
  });

  it('preserves ambiguous same-artifact activities through the hook and catalog merge', async () => {
    listModelDownloadsMock.mockResolvedValueOnce({ success: true, downloads: [
      progressOutcome({ downloadId: 'dl-first', libraryModelId: 'llm/org/model' }),
      progressOutcome({ downloadId: 'dl-second', libraryModelId: 'llm/org/model', status: 'paused' }),
    ] });
    const { result } = renderHook(() => useModelDownloads());
    await flushMicrotasks();
    const activities = buildDownloadingModels(result.current.downloadStatusByRepo);
    const merged = mergeLocalModelGroups([{ category: 'llm', models: [{
      id: 'llm/org/model', provenance: 'catalog', name: 'Catalog model', category: 'llm',
    }] }], activities);
    expect(activities).toHaveLength(2);
    expect(merged[0]?.models).toHaveLength(3);
    expect(merged[0]?.models.find(model => model.provenance === 'catalog')?.isDownloading).toBeUndefined();
    const secondKey = Object.entries(result.current.downloadStatusByRepo)
      .find(([, status]) => status.downloadId === 'dl-second')?.[0];
    expect(secondKey).toBeDefined();
    if (secondKey === undefined) return;
    await act(async () => { await result.current.resumeDownload(secondKey); });
    expect(resumeModelDownloadMock).toHaveBeenCalledWith('dl-second');
  });

  it('restores tracked downloads and repo-level errors on startup', async () => {
    listModelDownloadsMock.mockResolvedValueOnce({
      success: true,
      downloads: [
        {
          repoId: 'repo-paused',
          selectedArtifactId: 'repo-paused::Q4',
          downloadId: 'dl-paused',
          status: 'paused',
          progress: 42,
          modelName: 'Paused Model',
          modelType: 'checkpoint',
        },
        {
          repoId: 'repo-error',
          downloadId: 'dl-error',
          status: 'error',
          progress: 90,
          error: 'Disk full',
        },
        {
          repoId: 'repo-done',
          downloadId: 'dl-done',
          status: 'completed',
          progress: 100,
        },
      ],
    });

    const { result } = renderHook(() => useModelDownloads());

    await flushMicrotasks();

    expect(listModelDownloadsMock).toHaveBeenCalledTimes(1);
    expect(result.current.downloadStatusByRepo['repo-paused::Q4']).toEqual({
      downloadId: 'dl-paused',
      status: 'paused',
      progress: 42,
      repoId: 'repo-paused',
      selectedArtifactId: 'repo-paused::Q4',
      artifactId: undefined,
      downloadedBytes: undefined,
      totalBytes: undefined,
      speed: undefined,
      etaSeconds: undefined,
      modelName: 'Paused Model',
      modelType: 'checkpoint',
      retryAttempt: undefined,
      retryLimit: undefined,
      retrying: undefined,
      nextRetryDelaySeconds: undefined,
    });
    expect(result.current.downloadStatusByRepo['repo-error']).toEqual({
      downloadId: 'dl-error',
      status: 'error',
      progress: 90,
      repoId: 'repo-error',
      selectedArtifactId: undefined,
      artifactId: undefined,
      downloadedBytes: undefined,
      totalBytes: undefined,
      speed: undefined,
      etaSeconds: undefined,
      modelName: undefined,
      modelType: undefined,
      retryAttempt: undefined,
      retryLimit: undefined,
      retrying: undefined,
      nextRetryDelaySeconds: undefined,
    });
    expect(result.current.downloadErrors).toEqual({
      'repo-error': 'Disk full',
    });
    expect(result.current.hasActiveDownloads).toBe(false);
  });

  it('applies pushed backend progress updates after a local download begins', async () => {
    listModelDownloadsMock.mockResolvedValueOnce({
      success: true,
      downloads: [],
    });

    const { result } = renderHook(() => useModelDownloads());

    await flushMicrotasks();

    act(() => {
      result.current.startDownload('repo-a', 'dl-1', {
        modelName: 'Model A',
        modelType: 'checkpoint',
      });
    });

    expect(result.current.downloadStatusByRepo['repo-a']).toEqual({
      downloadId: 'dl-1',
      status: 'queued',
      progress: 0,
      repoId: 'repo-a',
      selectedArtifactId: undefined,
      artifactId: undefined,
      modelName: 'Model A',
      modelType: 'checkpoint',
    });

    await act(async () => {
      downloadUpdateCallback?.({
        cursor: 'download:2',
        snapshot: {
          cursor: 'download:2',
          revision: 2,
          downloads: [
            progressOutcome({
              repoId: 'repo-a',
              selectedArtifactId: 'repo-a::Q4',
              downloadId: 'dl-1',
              status: 'downloading',
              progress: 55,
              downloadedBytes: 550,
              totalBytes: 1000,
              speed: 32,
              etaSeconds: 14,
            }),
          ],
        },
        stale_cursor: false,
        snapshot_required: false,
      });
    });

    expect(listModelDownloadsMock).toHaveBeenCalledTimes(1);
    expect(result.current.downloadStatusByRepo['repo-a::Q4']).toEqual(
      expect.objectContaining({
        downloadId: 'dl-1',
        status: 'downloading',
        progress: 55,
        repoId: 'repo-a',
        selectedArtifactId: 'repo-a::Q4',
        downloadedBytes: 550,
        totalBytes: 1000,
        speed: 32,
        etaSeconds: 14,
      })
    );
    expect(result.current.hasActiveDownloads).toBe(true);
  });

  it('does not install a polling interval and unsubscribes on unmount', async () => {
    const setIntervalSpy = vi.spyOn(global, 'setInterval');
    const { unmount } = renderHook(() => useModelDownloads());

    await flushMicrotasks();

    expect(setIntervalSpy).not.toHaveBeenCalled();

    unmount();
    expect(unsubscribeMock).toHaveBeenCalledTimes(1);
    setIntervalSpy.mockRestore();
  });

  it('tracks same-repo artifact downloads independently and ignores duplicate same-ID starts', async () => {
    const { result } = renderHook(() => useModelDownloads());

    await flushMicrotasks();

    act(() => {
      result.current.startDownload('org/model::Q4', 'dl-q4', {
        repoId: 'org/model',
        artifactId: 'org/model::Q4',
        modelName: 'Model Q4',
      });
      result.current.startDownload('org/model::Q8', 'dl-q8', {
        repoId: 'org/model',
        selectedArtifactId: 'org/model::Q8',
        modelName: 'Model Q8',
      });
    });

    expect(result.current.downloadStatusByRepo['org/model::Q4']).toEqual({
      downloadId: 'dl-q4',
      status: 'queued',
      progress: 0,
      repoId: 'org/model',
      selectedArtifactId: undefined,
      artifactId: 'org/model::Q4',
      modelName: 'Model Q4',
      modelType: undefined,
    });
    expect(result.current.downloadStatusByRepo['org/model::Q8']).toEqual({
      downloadId: 'dl-q8',
      status: 'queued',
      progress: 0,
      repoId: 'org/model',
      selectedArtifactId: 'org/model::Q8',
      artifactId: undefined,
      modelName: 'Model Q8',
      modelType: undefined,
    });

    act(() => {
      result.current.startDownload('org/model::Q4', 'dl-q4', {
        repoId: 'org/model',
        artifactId: 'org/model::Q4',
        modelName: 'Duplicate Q4',
      });
    });

    expect(result.current.downloadStatusByRepo['org/model::Q4']).toEqual({
      downloadId: 'dl-q4',
      status: 'queued',
      progress: 0,
      repoId: 'org/model',
      selectedArtifactId: undefined,
      artifactId: 'org/model::Q4',
      modelName: 'Model Q4',
      modelType: undefined,
    });
    expect(Object.keys(result.current.downloadStatusByRepo).sort()).toEqual([
      'org/model::Q4',
      'org/model::Q8',
    ]);
  });

  it('clears stale errors, protects active downloads from duplicate starts, and routes pause/cancel actions', async () => {
    const { result } = renderHook(() => useModelDownloads());

    await flushMicrotasks();

    act(() => {
      result.current.setDownloadErrors({
        'repo-a': 'Old failure',
      });
      result.current.startDownload('repo-a', 'dl-1', {
        modelName: 'Model A',
      });
    });

    expect(result.current.downloadErrors).toEqual({});
    expect(result.current.downloadStatusByRepo['repo-a']).toEqual({
      downloadId: 'dl-1',
      status: 'queued',
      progress: 0,
      repoId: 'repo-a',
      selectedArtifactId: undefined,
      artifactId: undefined,
      modelName: 'Model A',
      modelType: undefined,
    });

    act(() => {
      result.current.startDownload('repo-a', 'dl-1', {
        modelName: 'Replacement Model',
      });
    });

    expect(result.current.downloadStatusByRepo['repo-a']).toEqual({
      downloadId: 'dl-1',
      status: 'queued',
      progress: 0,
      repoId: 'repo-a',
      selectedArtifactId: undefined,
      artifactId: undefined,
      modelName: 'Model A',
      modelType: undefined,
    });

    await act(async () => {
      await result.current.pauseDownload('repo-a');
    });

    expect(pauseModelDownloadMock).toHaveBeenCalledWith('dl-1');
    expect(result.current.downloadStatusByRepo['repo-a']).toEqual(
      expect.objectContaining({
        downloadId: 'dl-1',
        status: 'pausing',
      })
    );

    await act(async () => {
      await result.current.cancelDownload('repo-a');
    });

    expect(cancelModelDownloadMock).toHaveBeenCalledWith('dl-1');
    expect(result.current.downloadStatusByRepo['repo-a']).toEqual(
      expect.objectContaining({
        downloadId: 'dl-1',
        status: 'cancelling',
      })
    );
  });

  it('marks resumed downloads as failed when the backend resume request rejects', async () => {
    listModelDownloadsMock.mockResolvedValueOnce({
      success: true,
      downloads: [
        {
          repoId: 'repo-paused',
          downloadId: 'dl-paused',
          status: 'paused',
          progress: 25,
        },
      ],
    });
    resumeModelDownloadMock.mockResolvedValueOnce({
      success: false,
      error: 'Resume blocked',
    });

    const { result } = renderHook(() => useModelDownloads());

    await flushMicrotasks();

    act(() => {
      result.current.setDownloadErrors({
        'repo-paused': 'Old failure',
      });
    });

    await act(async () => {
      await result.current.resumeDownload('repo-paused');
    });

    expect(resumeModelDownloadMock).toHaveBeenCalledWith('dl-paused');
    expect(result.current.downloadStatusByRepo['repo-paused']).toEqual(
      expect.objectContaining({
        downloadId: 'dl-paused',
        status: 'error',
      })
    );
    expect(result.current.downloadErrors).toEqual({
      'repo-paused': 'Resume blocked',
    });
  });
});
