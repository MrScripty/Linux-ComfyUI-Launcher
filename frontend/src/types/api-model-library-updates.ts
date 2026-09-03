export type ModelFactFamily =
  | 'model_record'
  | 'metadata'
  | 'package_facts'
  | 'dependency_bindings'
  | 'validation'
  | 'search_index';

export type ModelLibraryChangeKind =
  | 'model_added'
  | 'model_removed'
  | 'metadata_modified'
  | 'package_facts_modified'
  | 'stale_facts_invalidated'
  | 'dependency_binding_modified';

export type ModelLibraryRefreshScope = 'summary' | 'detail' | 'summary_and_detail';

export interface ModelLibraryUpdateEvent {
  cursor: string;
  model_id: string;
  change_kind: ModelLibraryChangeKind;
  fact_family: ModelFactFamily;
  refresh_scope: ModelLibraryRefreshScope;
  selected_artifact_id?: string | null;
  producer_revision?: string | null;
}

export interface ModelLibraryUpdateFeed {
  cursor: string;
  events?: ModelLibraryUpdateEvent[];
  stale_cursor: boolean;
  snapshot_required: boolean;
}

export interface ModelLibraryUpdateNotification {
  cursor: string;
  events?: ModelLibraryUpdateEvent[];
  stale_cursor: boolean;
  snapshot_required: boolean;
}

const MODEL_FACT_FAMILIES: ReadonlySet<ModelFactFamily> = new Set([
  'model_record',
  'metadata',
  'package_facts',
  'dependency_bindings',
  'validation',
  'search_index',
]);

const MODEL_LIBRARY_CHANGE_KINDS: ReadonlySet<ModelLibraryChangeKind> = new Set([
  'model_added',
  'model_removed',
  'metadata_modified',
  'package_facts_modified',
  'stale_facts_invalidated',
  'dependency_binding_modified',
]);

const MODEL_LIBRARY_REFRESH_SCOPES: ReadonlySet<ModelLibraryRefreshScope> = new Set([
  'summary',
  'detail',
  'summary_and_detail',
]);

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function isNullableString(value: unknown): value is string | null | undefined {
  return value === undefined || value === null || typeof value === 'string';
}

export function isModelLibraryUpdateEvent(value: unknown): value is ModelLibraryUpdateEvent {
  if (!isRecord(value)) return false;
  return (
    typeof value['cursor'] === 'string' &&
    typeof value['model_id'] === 'string' &&
    MODEL_LIBRARY_CHANGE_KINDS.has(value['change_kind'] as ModelLibraryChangeKind) &&
    MODEL_FACT_FAMILIES.has(value['fact_family'] as ModelFactFamily) &&
    MODEL_LIBRARY_REFRESH_SCOPES.has(value['refresh_scope'] as ModelLibraryRefreshScope) &&
    isNullableString(value['selected_artifact_id']) &&
    isNullableString(value['producer_revision'])
  );
}

export function isModelLibraryUpdateNotification(
  value: unknown
): value is ModelLibraryUpdateNotification {
  if (!isRecord(value)) return false;
  if (
    typeof value['cursor'] !== 'string' ||
    typeof value['stale_cursor'] !== 'boolean' ||
    typeof value['snapshot_required'] !== 'boolean'
  ) {
    return false;
  }
  if (value['events'] === undefined) {
    return true;
  }
  return Array.isArray(value['events']) && value['events'].every(isModelLibraryUpdateEvent);
}
