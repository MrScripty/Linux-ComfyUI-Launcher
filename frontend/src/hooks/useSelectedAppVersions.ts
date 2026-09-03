import { useMemo } from 'react';
import { getAppVersionState, type AppVersionState } from '../utils/appVersionState';
import { useVersions, type UseVersionsResult } from './useVersions';

interface UseSelectedAppVersionsResult {
  appVersions: AppVersionState;
  installationProgress: UseVersionsResult['installationProgress'];
  llamaCppInstalledVersions: string[];
  ollamaInstalledVersions: string[];
  torchInstalledVersions: string[];
}

export function useSelectedAppVersions(selectedAppId: string | null): UseSelectedAppVersionsResult {
  const ollamaVersions = useVersions({
    appId: 'ollama',
    trackAvailableVersions: selectedAppId === 'ollama',
  });
  const llamaCppVersions = useVersions({
    appId: 'llama-cpp',
    trackAvailableVersions: selectedAppId === 'llama-cpp',
  });
  const torchVersions = useVersions({
    appId: 'torch',
    trackAvailableVersions: selectedAppId === 'torch',
  });

  const activeVersions = useMemo(() => {
    if (selectedAppId === 'ollama') return ollamaVersions;
    if (selectedAppId === 'llama-cpp') return llamaCppVersions;
    if (selectedAppId === 'torch') return torchVersions;
    return ollamaVersions;
  }, [selectedAppId, llamaCppVersions, ollamaVersions, torchVersions]);

  const appVersions = getAppVersionState(selectedAppId, activeVersions);

  return {
    appVersions,
    installationProgress: appVersions.installationProgress,
    llamaCppInstalledVersions: llamaCppVersions.installedVersions,
    ollamaInstalledVersions: ollamaVersions.installedVersions,
    torchInstalledVersions: torchVersions.installedVersions,
  };
}
