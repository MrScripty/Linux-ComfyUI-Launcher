import { Link2, Play, Save, SlidersHorizontal } from 'lucide-react';
import type { RuntimeProfileConfig } from '../../../types/api-runtime-profiles';
import { IconButton } from '../../ui';

interface RuntimeModelRowActionsProps {
  draftProfileId: string;
  hasDraftChange: boolean;
  isDraftProfileLoaded: boolean;
  isLinked: boolean;
  isQuickServing: boolean;
  isSavingRoute: boolean;
  modelId: string;
  modelName: string;
  profileName: string;
  profileSelectId: string;
  providerProfiles: RuntimeProfileConfig[];
  routeLabel: string;
  selectedProfile: RuntimeProfileConfig | undefined;
  startingServingTooltip?: string;
  getProfileOptionLabel?: (profile: RuntimeProfileConfig) => string;
  onOpenServeOptions: (profile: RuntimeProfileConfig) => void;
  onProfileIdChange: (profileId: string) => void;
  onQuickServe: (profile: RuntimeProfileConfig) => void;
  onSaveRoute: (modelId: string, profileId: string) => void;
  onToggleLink: (modelId: string) => void;
}

function quickServeTooltip({
  isDraftProfileLoaded,
  isQuickServing,
  profileName,
  startingServingTooltip,
}: Pick<
  RuntimeModelRowActionsProps,
  'isDraftProfileLoaded' | 'isQuickServing' | 'profileName' | 'startingServingTooltip'
>): string {
  if (isDraftProfileLoaded) {
    return 'Already loaded on selected profile';
  }
  if (isQuickServing && startingServingTooltip) {
    return startingServingTooltip;
  }
  return `Quick serve with selected ${profileName} profile`;
}

export function RuntimeModelRowActions({
  draftProfileId,
  hasDraftChange,
  isDraftProfileLoaded,
  isLinked,
  isQuickServing,
  isSavingRoute,
  modelId,
  modelName,
  profileName,
  profileSelectId,
  providerProfiles,
  routeLabel,
  selectedProfile,
  startingServingTooltip,
  getProfileOptionLabel = (profile) => profile.name,
  onOpenServeOptions,
  onProfileIdChange,
  onQuickServe,
  onSaveRoute,
  onToggleLink,
}: RuntimeModelRowActionsProps) {
  return (
    <div className="flex shrink-0 flex-wrap items-center justify-end gap-2 pt-0.5">
      <label className="sr-only" htmlFor={profileSelectId}>
        {profileName} profile for {modelName}
      </label>
      <select
        id={profileSelectId}
        value={draftProfileId}
        onChange={(event) => onProfileIdChange(event.target.value)}
        disabled={providerProfiles.length === 0}
        className="h-8 max-w-44 rounded border border-[hsl(var(--border-subtle))] bg-[hsl(var(--surface-high))] px-2 text-xs text-[hsl(var(--text-primary))]"
        aria-label={`${profileName} profile for ${modelName}`}
      >
        <option value="">
          {providerProfiles.length === 0 ? `No ${profileName} profiles` : routeLabel}
        </option>
        {providerProfiles.map((profile) => (
          <option key={profile.profile_id} value={profile.profile_id}>
            {getProfileOptionLabel(profile)}
          </option>
        ))}
      </select>
      <IconButton
        icon={<Save />}
        tooltip={`Save ${profileName} route`}
        onClick={() => onSaveRoute(modelId, draftProfileId)}
        disabled={!hasDraftChange || isSavingRoute}
        size="sm"
      />
      <IconButton
        icon={<Play />}
        tooltip={quickServeTooltip({
          isDraftProfileLoaded,
          isQuickServing,
          profileName,
          startingServingTooltip,
        })}
        onClick={selectedProfile ? () => onQuickServe(selectedProfile) : undefined}
        disabled={!selectedProfile || isQuickServing || isSavingRoute || isDraftProfileLoaded}
        size="sm"
      />
      <IconButton
        icon={<SlidersHorizontal />}
        tooltip="Serving options"
        onClick={selectedProfile ? () => onOpenServeOptions(selectedProfile) : undefined}
        disabled={!selectedProfile || isSavingRoute}
        size="sm"
      />
      <IconButton
        icon={<Link2 />}
        tooltip={isLinked ? `Unlink from ${profileName}` : `Link to ${profileName}`}
        onClick={() => onToggleLink(modelId)}
        size="sm"
      />
    </div>
  );
}
