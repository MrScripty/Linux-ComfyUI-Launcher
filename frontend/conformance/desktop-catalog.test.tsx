import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { runInNewContext } from 'node:vm';
import { useState } from 'react';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { LauncherRootRecoveryProvider } from '../src/hooks/useLauncherRootRecovery';
import { useModels } from '../src/hooks/useModels';
import { useModelLibraryActions } from '../src/hooks/useModelLibraryActions';
import { LocalModelsList } from '../src/components/LocalModelsList';
import { ValidationError } from '../src/errors';

const fixturePath = process.env['PUMAS_DESKTOP_CONTRACT_FIXTURES'];
if (!fixturePath) throw new ValidationError('Actual desktop producer fixtures are required; run test:desktop-contract.', 'producer-fixtures');
function isFixtureRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}
function readFixture(path: string): Record<string, unknown> {
  const value: unknown = JSON.parse(readFileSync(path, 'utf8'));
  if (!isFixtureRecord(value)) {
    throw new ValidationError('Invalid producer fixture envelope.', 'producer-fixtures');
  }
  return value;
}
const fixture = readFixture(fixturePath);
const preload = readFileSync(resolve('../electron/dist/preload.js'), 'utf8');

function installActualPreload() {
  const requests: Array<{ method: string; params: unknown }> = [];
  const module = { exports: {} };
  const electron = {
    contextBridge: {
      exposeInMainWorld: (name: string, bridge: typeof window.electronAPI) => {
        expect(name).toBe('electronAPI');
        window.electronAPI = bridge;
      },
    },
    ipcRenderer: {
      on: () => undefined,
      removeListener: () => undefined,
      sendSync: (channel: string) => {
        expect(channel).toBe('launcher:getRootBootstrap');
        return { status: 'ready', selectionAction: 'select-library', libraryScopeId: null };
      },
      invoke: async (channel: string, method: string, params: unknown) => {
        if (channel === 'launcher-root:presentation-committed') return undefined;
        expect(channel).toBe('api:call');
        const requestParams: unknown = JSON.parse(JSON.stringify(params));
        requests.push({ method, params: requestParams });
        if (method === 'get_models') return fixture['models'];
        if (method === 'search_models_fts') return fixture['search'];
        if (method === 'resume_partial_download') return fixture['recovery_outcome'];
        throw new ValidationError(`Unprovided fixture operation: ${method}`, 'producer-fixtures');
      },
    },
    webUtils: { getPathForFile: () => '' },
  };
  runInNewContext(preload, {
    exports: module.exports, module,
    require: (name: string) => {
      expect(name).toBe('electron');
      return electron;
    },
  }, { filename: 'electron/dist/preload.js' });
  expect(window.electronAPI).toBeDefined();
  return requests;
}

type StartDownload = Parameters<typeof useModelLibraryActions>[0]['startDownload'];

function Library({ onStarted }: { onStarted: StartDownload }) {
  const { modelGroups, libraryLoadStatus } = useModels();
  const [downloadErrors, setDownloadErrors] = useState<Record<string, string>>({});
  const actions = useModelLibraryActions({
    setDownloadErrors, startDownload: onStarted,
  });
  return <>
    <div role="status">{libraryLoadStatus}</div>
    <LocalModelsList
      modelGroups={modelGroups} totalModels={modelGroups.flatMap((group) => group.models).length}
      starredModels={new Set()} excludedModels={new Set()} selectedAppId={null} hasFilters={false}
      onToggleStar={() => undefined} onToggleLink={() => undefined}
      relatedModelsById={actions.relatedModelsById} expandedRelated={actions.expandedRelated}
      onToggleRelated={actions.handleToggleRelated} onOpenRelatedUrl={actions.openRemoteUrl}
      onRecoverPartialDownload={actions.handleRecoverPartialDownload}
      recoveringPartialModelIds={actions.recoveringPartialModelIds} downloadErrors={downloadErrors}
    />
  </>;
}

describe('actual Rust catalog through bundled preload and renderer', () => {
  afterEach(() => { window.electronAPI = undefined; });

  it('renders complete, partial, and duplicate facts and sends the exact producer-admitted recovery ticket', async () => {
    const requests = installActualPreload();
    const onStarted = vi.fn<StartDownload>();
    render(<LauncherRootRecoveryProvider><Library onStarted={onStarted} /></LauncherRootRecoveryProvider>);
    await waitFor(() => expect(screen.getByRole('status')).toHaveTextContent('ready'));
    expect(screen.getByText('complete', { exact: true })).toBeVisible();
    expect(screen.getByText('partial', { exact: true })).toBeVisible();
    expect(screen.getByText('PARTIAL 50%')).toBeVisible();
    expect(screen.getAllByText('ISSUE')).toHaveLength(2);
    expect(screen.getByRole('button', { name: 'Show related' })).toBeEnabled();
    const resume = screen.getByRole('button', { name: 'Resume partial download (50%)' });
    fireEvent.click(resume);
    await waitFor(() => expect(onStarted).toHaveBeenCalledOnce());
    expect(requests.find((request) => request.method === 'resume_partial_download')?.params)
      .toEqual(fixture['recovery_request']);
    expect(onStarted).toHaveBeenCalledWith('fixture-download', 'fixture-download', {
      modelName: 'partial', modelType: 'llm', repoId: 'example/model', selectedArtifactId: 'example/model::Q4',
    });
  });

  it('accepts the same typed catalog projection from the actual search producer', async () => {
    installActualPreload();
    const bridge = window.electronAPI;
    if (!bridge) throw new ValidationError('Preload did not expose its bridge.', 'preload');
    const response = await bridge.search_models_fts('', 100, 0);
    expect(response.success).toBe(true);
    expect(response.models).toHaveLength(4);
    expect(response.models.find((model) => model.id === 'llm/example/partial')?.artifact.state).toBe('partial');
  });
});
