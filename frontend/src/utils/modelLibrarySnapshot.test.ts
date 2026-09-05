import { beforeEach, describe, expect, it } from 'vitest';
import { MODEL_LIBRARY_SNAPSHOT_KEY, readModelLibrarySnapshot, writeModelLibrarySnapshot } from './modelLibrarySnapshot';

const scope = `display-v1:${'a'.repeat(64)}`;

describe('root-scoped display-only library snapshot', () => {
  beforeEach(() => localStorage.clear());

  it('reopens display fields without model paths or download authority', () => {
    expect(writeModelLibrarySnapshot([{ category: 'llm', models: [{
      id: 'llm/example', name: 'Example', category: 'llm', format: 'gguf',
      modelDir: '/models/example', path: 'llm/example', repoId: 'org/example',
      selectedArtifactId: 'artifact', isDownloading: true, downloadKey: 'download',
    }] }], scope)).toBe(true);
    expect(readModelLibrarySnapshot(scope)).toEqual([{ category: 'llm', models: [{
      id: 'llm/example', name: 'Example', category: 'llm', format: 'gguf', provenance: 'cached',
    }] }]);
    expect(localStorage.getItem(MODEL_LIBRARY_SNAPSHOT_KEY)).not.toContain('/models/example');
  });

  it('rejects another root, missing scope, and unscoped v1 data', () => {
    writeModelLibrarySnapshot([{ category: 'llm', models: [{ id: 'one', name: 'One', category: 'llm' }] }], scope);
    expect(readModelLibrarySnapshot(`display-v1:${'b'.repeat(64)}`)).toEqual([]);
    expect(readModelLibrarySnapshot(null)).toEqual([]);
    localStorage.setItem('pumas:model-library:v1', JSON.stringify({ modelGroups: [] }));
    expect(readModelLibrarySnapshot(scope)).toEqual([]);
    expect(localStorage.getItem('pumas:model-library:v1')).toBeNull();
  });

  it('rejects a snapshot containing an unknown capability field', () => {
    writeModelLibrarySnapshot([{ category: 'llm', models: [{ id: 'one', name: 'One', category: 'llm' }] }], scope);
    const value = { libraryScopeId: scope, modelGroups: [{ category: 'llm', models: [{
      id: 'one', name: 'One', category: 'llm', recoveryToken: 'injected',
    }] }] };
    localStorage.setItem(MODEL_LIBRARY_SNAPSHOT_KEY, JSON.stringify(value));
    expect(readModelLibrarySnapshot(scope)).toEqual([]);
    expect(localStorage.getItem(MODEL_LIBRARY_SNAPSHOT_KEY)).toBeNull();
  });
});
