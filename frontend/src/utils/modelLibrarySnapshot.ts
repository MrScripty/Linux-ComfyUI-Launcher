import type { ModelCategory, ModelInfo } from '../types/apps';

export const MODEL_LIBRARY_SNAPSHOT_KEY = 'pumas:model-library:v2';
const RETIRED_SNAPSHOT_KEY = 'pumas:model-library:v1';

const DISPLAY_STRINGS = ['format', 'quant', 'date', 'integrityIssueMessage'] as const;
const DISPLAY_NUMBERS = ['size', 'downloadProgress', 'dependencyCount'] as const;
const DISPLAY_FLAGS = ['isPartialDownload', 'hasDependencies', 'hasIntegrityIssue'] as const;
const DISPLAY_KEYS = new Set<string>([
  'id', 'name', 'category', ...DISPLAY_STRINGS, ...DISPLAY_NUMBERS, ...DISPLAY_FLAGS,
]);

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function isDisplayString(value: unknown): value is string {
  return typeof value === 'string' && value.trim().length > 0 && value.length <= 4096;
}

function isScope(value: unknown): value is string {
  return typeof value === 'string' && /^display-v1:[a-f0-9]{64}$/.test(value);
}

function decodeDisplayModel(value: unknown, category: string): ModelInfo | null {
  if (!isRecord(value) || Object.keys(value).some((key) => !DISPLAY_KEYS.has(key)) ||
    !isDisplayString(value['id']) || !isDisplayString(value['name']) || value['category'] !== category) {
    return null;
  }
  if (DISPLAY_STRINGS.some((key) => value[key] !== undefined && !isDisplayString(value[key])) ||
    DISPLAY_FLAGS.some((key) => value[key] !== undefined && typeof value[key] !== 'boolean') ||
    DISPLAY_NUMBERS.some((key) => value[key] !== undefined &&
      (typeof value[key] !== 'number' || !Number.isFinite(value[key]) || value[key] < 0))) {
    return null;
  }
  if (typeof value['downloadProgress'] === 'number' && value['downloadProgress'] >= 1) return null;
  if (typeof value['size'] === 'number' && !Number.isSafeInteger(value['size'])) return null;
  if (typeof value['dependencyCount'] === 'number' && !Number.isSafeInteger(value['dependencyCount'])) return null;
  return { ...value, id: value['id'], name: value['name'], category, provenance: 'cached' };
}

function decodeGroups(value: unknown): ModelCategory[] | null {
  if (!Array.isArray(value)) return null;
  const groups: ModelCategory[] = [];
  const ids = new Set<string>();
  const categories = new Set<string>();
  for (const group of value) {
    if (!isRecord(group) || Object.keys(group).some((key) => key !== 'category' && key !== 'models') ||
      !isDisplayString(group['category']) || !Array.isArray(group['models']) || categories.has(group['category'])) {
      return null;
    }
    categories.add(group['category']);
    const models: ModelInfo[] = [];
    for (const value of group['models']) {
      const model = decodeDisplayModel(value, group['category']);
      if (!model || ids.has(model.id)) return null;
      ids.add(model.id);
      models.push(model);
    }
    groups.push({ category: group['category'], models });
  }
  return groups;
}

function displayOnly(model: ModelInfo): Record<string, unknown> {
  const display: Record<string, unknown> = { id: model.id, name: model.name, category: model.category };
  for (const key of [...DISPLAY_STRINGS, ...DISPLAY_NUMBERS, ...DISPLAY_FLAGS]) {
    if (model[key] !== undefined) display[key] = model[key];
  }
  return display;
}

export function toDisplayOnlyModelGroups(modelGroups: ModelCategory[]): ModelCategory[] {
  return modelGroups.map((group) => ({
    category: group.category,
    models: group.models.map((model) => ({
      ...displayOnly(model),
      id: model.id,
      name: model.name,
      category: model.category,
      provenance: 'cached',
    })),
  }));
}

function getLocalStorage(): Storage | null {
  try {
    return typeof window === 'undefined' ? null : window.localStorage;
  } catch {
    return null;
  }
}

/** Disposable display cache only: never a source of paths, activity, or action authority. */
export function readModelLibrarySnapshot(libraryScopeId: string | null): ModelCategory[] {
  const storage = getLocalStorage();
  if (!storage) return [];
  try {
    storage.removeItem(RETIRED_SNAPSHOT_KEY);
    const serialized = storage.getItem(MODEL_LIBRARY_SNAPSHOT_KEY);
    if (!serialized) return [];
    const parsed: unknown = JSON.parse(serialized);
    const groups = isScope(libraryScopeId) && isRecord(parsed) &&
      Object.keys(parsed).length === 2 && parsed['libraryScopeId'] === libraryScopeId
      ? decodeGroups(parsed['modelGroups']) : null;
    if (groups) return groups;
    storage.removeItem(MODEL_LIBRARY_SNAPSHOT_KEY);
  } catch {
    try { storage.removeItem(MODEL_LIBRARY_SNAPSHOT_KEY); } catch { /* Storage may be unavailable. */ }
  }
  return [];
}

export function writeModelLibrarySnapshot(modelGroups: ModelCategory[], libraryScopeId: string | null): boolean {
  const storage = getLocalStorage();
  if (!storage || !isScope(libraryScopeId)) return false;
  try {
    const groups = modelGroups.map((group) => ({ category: group.category, models: group.models.map(displayOnly) }));
    if (!decodeGroups(groups)) return false;
    storage.removeItem(RETIRED_SNAPSHOT_KEY);
    storage.setItem(MODEL_LIBRARY_SNAPSHOT_KEY, JSON.stringify({ libraryScopeId, modelGroups: groups }));
    return true;
  } catch {
    return false;
  }
}
