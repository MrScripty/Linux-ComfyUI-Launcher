import type { ModelCategory, ModelInfo } from '../types/apps';

export const MODEL_LIBRARY_SNAPSHOT_KEY = 'pumas:model-library:v1';

interface ModelLibrarySnapshot {
  modelGroups: ModelCategory[];
}

const OPTIONAL_STRING_FIELDS = [
  'path',
  'modelDir',
  'format',
  'quant',
  'date',
  'downloadKey',
  'downloadRepoId',
  'convertedFrom',
  'repoId',
  'integrityIssueMessage',
] as const;

const OPTIONAL_NULLABLE_STRING_FIELDS = [
  'downloadSelectedArtifactId',
  'downloadArtifactId',
  'selectedArtifactId',
  'selectedArtifactQuant',
] as const;

const OPTIONAL_BOOLEAN_FIELDS = [
  'starred',
  'relatedAvailable',
  'isPartialDownload',
  'isDownloading',
  'wasDequantized',
  'hasDependencies',
  'hasIntegrityIssue',
] as const;

const OPTIONAL_NUMBER_FIELDS = [
  'size',
  'downloadProgress',
  'downloadTotalBytes',
  'dependencyCount',
] as const;

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function optionalFieldsMatch(
  value: Record<string, unknown>,
  fields: readonly string[],
  predicate: (fieldValue: unknown) => boolean
): boolean {
  return fields.every((field) => value[field] === undefined || predicate(value[field]));
}

function isStringArray(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((item) => typeof item === 'string');
}

function isCachedModel(value: unknown): value is ModelInfo {
  if (!isRecord(value)) {
    return false;
  }

  const downloadStatus = value['downloadStatus'];
  const primaryFormat = value['primaryFormat'];

  return (
    typeof value['id'] === 'string' &&
    typeof value['name'] === 'string' &&
    typeof value['category'] === 'string' &&
    optionalFieldsMatch(value, OPTIONAL_STRING_FIELDS, (field) => typeof field === 'string') &&
    optionalFieldsMatch(
      value,
      OPTIONAL_NULLABLE_STRING_FIELDS,
      (field) => field === null || typeof field === 'string'
    ) &&
    optionalFieldsMatch(value, OPTIONAL_BOOLEAN_FIELDS, (field) => typeof field === 'boolean') &&
    optionalFieldsMatch(
      value,
      OPTIONAL_NUMBER_FIELDS,
      (field) => typeof field === 'number' && Number.isFinite(field)
    ) &&
    (value['linkedApps'] === undefined || isStringArray(value['linkedApps'])) &&
    (value['selectedArtifactFiles'] === undefined ||
      isStringArray(value['selectedArtifactFiles'])) &&
    (downloadStatus === undefined ||
      ['queued', 'downloading', 'pausing', 'paused', 'cancelling', 'error'].includes(
        downloadStatus as string
      )) &&
    (primaryFormat === undefined ||
      primaryFormat === 'gguf' ||
      primaryFormat === 'safetensors' ||
      primaryFormat === 'onnx')
  );
}

function isCachedCategory(value: unknown): value is ModelCategory {
  return (
    isRecord(value) &&
    typeof value['category'] === 'string' &&
    Array.isArray(value['models']) &&
    value['models'].every(isCachedModel)
  );
}

function getLocalStorage(): Storage | null {
  if (typeof window === 'undefined') {
    return null;
  }

  try {
    return window.localStorage;
  } catch {
    return null;
  }
}

export function readModelLibrarySnapshot(): ModelCategory[] {
  const storage = getLocalStorage();
  if (!storage) {
    return [];
  }

  try {
    const serialized = storage.getItem(MODEL_LIBRARY_SNAPSHOT_KEY);
    if (!serialized) {
      return [];
    }

    const parsed: unknown = JSON.parse(serialized);
    if (!isRecord(parsed) || !Array.isArray(parsed['modelGroups'])) {
      return [];
    }

    return parsed['modelGroups'].every(isCachedCategory) ? parsed['modelGroups'] : [];
  } catch {
    return [];
  }
}

export function writeModelLibrarySnapshot(modelGroups: ModelCategory[]): boolean {
  const storage = getLocalStorage();
  if (!storage) {
    return false;
  }

  try {
    const snapshot: ModelLibrarySnapshot = { modelGroups };
    storage.setItem(MODEL_LIBRARY_SNAPSHOT_KEY, JSON.stringify(snapshot));
    return true;
  } catch {
    return false;
  }
}
