import { useEffect, useState } from 'react';
import { Star } from 'lucide-react';
import type { RuntimeProfileConfig } from '../../../types/api-runtime-profiles';
import type { ServedModelStatus } from '../../../types/api-serving';
import { LocalModelMetadataSummary } from '../../LocalModelMetadataSummary';
import { LocalModelNameButton } from '../../LocalModelNameButton';
import { IconButton, ListItem, ListItemContent } from '../../ui';
import type { OnnxRuntimeModelRowViewModel } from './onnxRuntimeLibraryViewModels';
import { RuntimeModelRowActions } from './RuntimeModelRowActions';

interface OnnxRuntimeModelRowProps {
  excludedModels: Set<string>;
  isQuickServing: boolean;
  isSavingRoute: boolean;
  providerProfiles: RuntimeProfileConfig[];
  quickServeFeedback: {
    kind: 'error' | 'success';
    message: string;
  } | null;
  row: OnnxRuntimeModelRowViewModel;
  starredModels: Set<string>;
  onOpenMetadata: (modelId: string, modelName: string) => void;
  onOpenServeOptions: (
    row: OnnxRuntimeModelRowViewModel,
    profile: RuntimeProfileConfig,
    shouldPersistRoute: boolean
  ) => void;
  onQuickServe: (
    row: OnnxRuntimeModelRowViewModel,
    profile: RuntimeProfileConfig,
    shouldPersistRoute: boolean
  ) => void;
  onSaveRoute: (modelId: string, profileId: string) => void;
  onToggleLink: (modelId: string) => void;
  onToggleStar: (modelId: string) => void;
}

function routeLabel(row: OnnxRuntimeModelRowViewModel): string {
  if (row.routeState === 'missing_profile') {
    return 'Missing profile';
  }
  return row.selectedProfile?.name ?? 'No profile';
}

function OnnxRuntimeStatusBadges({ row }: { row: OnnxRuntimeModelRowViewModel }) {
  const loadedStatuses = row.servedStatuses.filter(
    (status: ServedModelStatus) => status.load_state === 'loaded'
  );
  const failedStatus = row.selectedServedStatus?.load_state === 'failed'
    ? row.selectedServedStatus
    : row.servedStatuses.find((status) => status.load_state === 'failed') ?? null;

  return (
    <>
      <span className="rounded bg-[hsl(var(--surface-low)/0.55)] px-1.5 py-0.5 text-[10px] font-medium uppercase text-[hsl(var(--text-secondary))]">
        ONNX
      </span>
      {row.routeState === 'missing_profile' && (
        <span className="rounded bg-[hsl(var(--accent-error)/0.14)] px-1.5 py-0.5 text-[10px] font-medium uppercase text-[hsl(var(--accent-error))]">
          Missing profile
        </span>
      )}
      {loadedStatuses.length > 0 && (
        <span
          className="rounded bg-[hsl(var(--accent-success)/0.14)] px-1.5 py-0.5 text-[10px] font-medium uppercase text-[hsl(var(--accent-success))]"
          title={loadedStatuses[0]?.endpoint_url ?? undefined}
        >
          Loaded {loadedStatuses.length}
        </span>
      )}
      {failedStatus && (
        <span
          className="rounded bg-[hsl(var(--accent-error)/0.14)] px-1.5 py-0.5 text-[10px] font-medium uppercase text-[hsl(var(--accent-error))]"
          title={failedStatus.last_error?.message ?? undefined}
        >
          Failed
        </span>
      )}
    </>
  );
}

export function OnnxRuntimeModelRow({
  excludedModels,
  isQuickServing,
  isSavingRoute,
  providerProfiles,
  quickServeFeedback,
  row,
  starredModels,
  onOpenMetadata,
  onOpenServeOptions,
  onQuickServe,
  onSaveRoute,
  onToggleLink,
  onToggleStar,
}: OnnxRuntimeModelRowProps) {
  const isStarred = starredModels.has(row.model.id) || Boolean(row.model.starred);
  const isLinked = row.model.linkedApps?.includes('onnx-runtime') ?? false;
  const isExcluded = excludedModels.has(row.model.id);
  const [draftProfileId, setDraftProfileId] = useState(row.route?.profile_id ?? '');
  const hasDraftChange = draftProfileId !== (row.route?.profile_id ?? '');
  const draftProfile = providerProfiles.find((profile) => profile.profile_id === draftProfileId);
  const isDraftProfileLoaded = row.servedStatuses.some(
    (status) => status.profile_id === draftProfileId && status.load_state === 'loaded'
  );

  useEffect(() => {
    setDraftProfileId(row.route?.profile_id ?? '');
  }, [row.route?.profile_id]);

  return (
    <ListItem highlighted={isLinked} className={isExcluded ? 'opacity-60' : ''}>
      <ListItemContent className="items-start">
        <div className="flex min-w-0 flex-1 items-start gap-2">
          <IconButton
            icon={<Star fill={isStarred ? 'currentColor' : 'none'} />}
            tooltip={isStarred ? 'Unstar' : 'Star'}
            onClick={() => onToggleStar(row.model.id)}
            size="sm"
          />
          <div className="min-w-0 flex-1">
            <div className="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1">
              <LocalModelNameButton
                downloadProgress={row.model.downloadProgress}
                modelId={row.model.id}
                modelName={row.model.name}
                isDownloading={false}
                isPartialDownload={Boolean(row.model.isPartialDownload)}
                isLinked={isLinked}
                wasDequantized={row.model.wasDequantized}
                hasIntegrityIssue={Boolean(row.model.hasIntegrityIssue)}
                integrityIssueMessage={row.model.integrityIssueMessage}
                onOpenMetadata={() => onOpenMetadata(row.model.id, row.model.name)}
              />
              <OnnxRuntimeStatusBadges row={row} />
            </div>
            <LocalModelMetadataSummary
              format={row.model.primaryFormat ?? row.model.format}
              quant={row.model.quant}
              size={row.model.size}
              hasDependencies={row.model.hasDependencies}
              dependencyCount={row.model.dependencyCount}
            />
            {quickServeFeedback && (
              <div
                className={
                  quickServeFeedback.kind === 'error'
                    ? 'mt-1 text-xs text-[hsl(var(--accent-error))]'
                    : 'mt-1 text-xs text-[hsl(var(--accent-success))]'
                }
              >
                {quickServeFeedback.message}
              </div>
            )}
          </div>
        </div>
        <RuntimeModelRowActions
          draftProfileId={draftProfileId}
          hasDraftChange={hasDraftChange}
          isDraftProfileLoaded={isDraftProfileLoaded}
          isLinked={isLinked}
          isQuickServing={isQuickServing}
          isSavingRoute={isSavingRoute}
          modelId={row.model.id}
          modelName={row.model.name}
          profileName="ONNX Runtime"
          profileSelectId={`onnx-profile-${row.model.id}`}
          providerProfiles={providerProfiles}
          routeLabel={routeLabel(row)}
          selectedProfile={draftProfile}
          startingServingTooltip="Starting ONNX Runtime serving"
          onOpenServeOptions={(profile) => onOpenServeOptions(row, profile, hasDraftChange)}
          onProfileIdChange={setDraftProfileId}
          onQuickServe={(profile) => onQuickServe(row, profile, hasDraftChange)}
          onSaveRoute={onSaveRoute}
          onToggleLink={onToggleLink}
        />
      </ListItemContent>
    </ListItem>
  );
}
