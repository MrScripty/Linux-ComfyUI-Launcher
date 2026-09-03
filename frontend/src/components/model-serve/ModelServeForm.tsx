import type { RefObject } from 'react';
import type {
  RuntimeDeviceMode,
  RuntimeProfileConfig,
  RuntimeProfileStatus,
} from '../../types/api-runtime-profiles';
import type { ModelInfo } from '../../types/apps';
import type { ModelServeControls, ModelServeFormState } from './modelServeHelpers';
import { PlacementControls } from './ModelServePlacementControls';

type ModelServeFormProps = {
  controls: ModelServeControls;
  formState: ModelServeFormState;
  model: ModelInfo;
  onProfileIdChange: (value: string) => void;
  profileId: string;
  profileSelectRef: RefObject<HTMLSelectElement | null>;
  profiles: RuntimeProfileConfig[];
  selectedProfile: RuntimeProfileConfig | undefined;
  selectedStatus: RuntimeProfileStatus | null;
  serveBlockReason: string | null;
  aliasRequired: boolean;
  aliasError: string | null;
  setContextSize: (value: string) => void;
  setDeviceId: (value: string) => void;
  setDeviceMode: (value: RuntimeDeviceMode) => void;
  setGpuLayers: (value: string) => void;
  setKeepLoaded: (value: boolean) => void;
  setModelAlias: (value: string) => void;
  setTensorSplit: (value: string) => void;
};

export function ModelServeForm({
  controls,
  formState,
  model,
  onProfileIdChange,
  profileId,
  profileSelectRef,
  profiles,
  selectedProfile,
  selectedStatus,
  serveBlockReason,
  aliasRequired,
  aliasError,
  setContextSize,
  setDeviceId,
  setDeviceMode,
  setGpuLayers,
  setKeepLoaded,
  setModelAlias,
  setTensorSplit,
}: ModelServeFormProps) {
  return (
    <div className="grid gap-3">
      <ProfileSelect
        onProfileIdChange={onProfileIdChange}
        profileId={profileId}
        profileSelectRef={profileSelectRef}
        profiles={profiles}
      />
      <ServeReadiness
        model={model}
        selectedProfile={selectedProfile}
        serveBlockReason={serveBlockReason}
      />
      <ProfileSummary selectedProfile={selectedProfile} selectedStatus={selectedStatus} />
      <AliasControls
        aliasError={aliasError}
        aliasRequired={aliasRequired}
        formState={formState}
        setModelAlias={setModelAlias}
      />
      <PlacementControls
        controls={controls}
        formState={formState}
        setContextSize={setContextSize}
        setDeviceId={setDeviceId}
        setDeviceMode={setDeviceMode}
        setGpuLayers={setGpuLayers}
        setTensorSplit={setTensorSplit}
      />
      <label className="flex items-center gap-2 text-xs text-[hsl(var(--text-secondary))]">
        <input
          type="checkbox"
          checked={formState.keepLoaded}
          onChange={(event) => setKeepLoaded(event.target.checked)}
        />
        Keep loaded
      </label>
    </div>
  );
}

function AliasControls({
  aliasError,
  aliasRequired,
  formState,
  setModelAlias,
}: {
  aliasError: string | null;
  aliasRequired: boolean;
  formState: ModelServeFormState;
  setModelAlias: (value: string) => void;
}) {
  if (!aliasRequired && !formState.modelAlias.trim()) {
    return null;
  }

  return (
    <label className="grid gap-1 text-xs text-[hsl(var(--text-secondary))]">
      Gateway alias
      <input
        value={formState.modelAlias}
        onChange={(event) => setModelAlias(event.target.value)}
        placeholder="unique-model-alias"
        className="rounded border border-[hsl(var(--border-default))] bg-[hsl(var(--surface-base))] px-2 py-1.5 text-sm text-[hsl(var(--text-primary))]"
      />
      {aliasRequired && (
        <span className="text-[hsl(var(--text-muted))]">
          This model is already served on another profile. Use a unique alias for this instance.
        </span>
      )}
      {aliasError && (
        <span className="text-[hsl(var(--accent-error))]">{aliasError}</span>
      )}
    </label>
  );
}

function ProfileSelect({
  onProfileIdChange,
  profileId,
  profileSelectRef,
  profiles,
}: {
  onProfileIdChange: (value: string) => void;
  profileId: string;
  profileSelectRef: RefObject<HTMLSelectElement | null>;
  profiles: RuntimeProfileConfig[];
}) {
  return (
    <label className="grid gap-1 text-xs text-[hsl(var(--text-secondary))]">
      Runtime target
      <select
        ref={profileSelectRef}
        value={profileId}
        onChange={(event) => onProfileIdChange(event.target.value)}
        className="rounded border border-[hsl(var(--border-default))] bg-[hsl(var(--surface-base))] px-2 py-1.5 text-sm text-[hsl(var(--text-primary))]"
      >
        {profiles.map((profile) => (
          <option key={profile.profile_id} value={profile.profile_id}>
            {profile.name}
          </option>
        ))}
      </select>
    </label>
  );
}

function ServeReadiness({
  model,
  selectedProfile,
  serveBlockReason,
}: {
  model: ModelInfo;
  selectedProfile: RuntimeProfileConfig | undefined;
  serveBlockReason: string | null;
}) {
  return (
    <div className="rounded border border-[hsl(var(--border-default))] bg-[hsl(var(--launcher-bg-secondary))] px-3 py-2 text-xs text-[hsl(var(--text-secondary))]">
      {serveBlockReason
        ? `Cannot serve yet: ${serveBlockReason}`
        : `Ready to serve ${model.name} with ${selectedProfile?.name ?? 'the selected runtime target'}.`}
    </div>
  );
}

function ProfileSummary({
  selectedProfile,
  selectedStatus,
}: {
  selectedProfile: RuntimeProfileConfig | undefined;
  selectedStatus: RuntimeProfileStatus | null;
}) {
  return (
    <div className="grid grid-cols-3 gap-3 text-xs">
      <SummaryField label="Provider" value={selectedProfile?.provider ?? 'none'} />
      <SummaryField label="Serving mode" value={selectedProfile?.provider_mode ?? 'none'} />
      <SummaryField label="State" value={selectedStatus?.state ?? 'unknown'} />
    </div>
  );
}

function SummaryField({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <div className="text-[hsl(var(--text-tertiary))]">{label}</div>
      <div className="mt-1 text-[hsl(var(--text-primary))]">{value}</div>
    </div>
  );
}
