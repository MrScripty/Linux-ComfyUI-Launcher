/**
 * Owns dialog-only presentation state derived from manager-owned progress.
 */

import { useCallback, useEffect, useRef, useState } from 'react';
import type { InstallationProgress } from './useVersions';

interface UseInstallationProgressOptions {
  externalProgress?: InstallationProgress | null;
}

interface UseInstallationProgressResult {
  progress: InstallationProgress | null;
  cancellationNotice: string | null;
  failedInstall: { tag: string; log: string | null } | null;
  setFailedInstall: (value: { tag: string; log: string | null } | null) => void;
  showCancellationNotice: () => void;
}

export function useInstallationProgress({
  externalProgress,
}: UseInstallationProgressOptions): UseInstallationProgressResult {
  const [progress, setProgress] = useState<InstallationProgress | null>(externalProgress ?? null);
  const [cancellationNotice, setCancellationNotice] = useState<string | null>(null);
  const [failedInstall, setFailedInstall] = useState<{ tag: string; log: string | null } | null>(null);
  const noticeTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const showCancellationNotice = useCallback(() => {
    if (noticeTimeoutRef.current) {
      clearTimeout(noticeTimeoutRef.current);
    }

    setCancellationNotice('Installation canceled');
    noticeTimeoutRef.current = setTimeout(() => {
      noticeTimeoutRef.current = null;
      setCancellationNotice(null);
    }, 3000);
  }, []);

  useEffect(() => {
    setProgress(externalProgress ?? null);
    if (!externalProgress) {
      return;
    }

    const isCancelled = externalProgress.error?.toLowerCase().includes('cancel');
    if (externalProgress.completed_at && isCancelled) {
      showCancellationNotice();
      return;
    }

    if (externalProgress.completed_at && !externalProgress.success && externalProgress.tag) {
      const nextFailedInstall = {
        tag: externalProgress.tag,
        log: externalProgress.log_path || null,
      };
      setFailedInstall((current) => (
        current?.tag === nextFailedInstall.tag && current.log === nextFailedInstall.log
          ? current
          : nextFailedInstall
      ));
      return;
    }

    if (externalProgress.completed_at && externalProgress.success && externalProgress.tag) {
      setFailedInstall((current) => (
        current?.tag === externalProgress.tag ? null : current
      ));
    }
  }, [externalProgress, showCancellationNotice]);

  useEffect(() => () => {
    if (noticeTimeoutRef.current) {
      clearTimeout(noticeTimeoutRef.current);
      noticeTimeoutRef.current = null;
    }
  }, []);

  return {
    progress,
    cancellationNotice,
    failedInstall,
    setFailedInstall,
    showCancellationNotice,
  };
}
