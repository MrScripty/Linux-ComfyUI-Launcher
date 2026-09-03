import type { RuntimeDeviceMode } from '../../types/api-runtime-profiles';
import {
  DEVICE_OPTIONS,
  type ModelServeControls,
  type ModelServeFormState,
} from './modelServeHelpers';

interface PlacementControlsProps {
  controls: ModelServeControls;
  formState: ModelServeFormState;
  setContextSize: (value: string) => void;
  setDeviceId: (value: string) => void;
  setDeviceMode: (value: RuntimeDeviceMode) => void;
  setGpuLayers: (value: string) => void;
  setTensorSplit: (value: string) => void;
}

export function PlacementControls({
  controls,
  formState,
  setContextSize,
  setDeviceId,
  setDeviceMode,
  setGpuLayers,
  setTensorSplit,
}: PlacementControlsProps) {
  return (
    <>
      {controls.showDeviceControls ? (
        <DeviceControls
          formState={formState}
          setDeviceId={setDeviceId}
          setDeviceMode={setDeviceMode}
          showDeviceId={controls.showDeviceId}
        />
      ) : (
        <div className="rounded border border-[hsl(var(--border-default))] px-3 py-2 text-xs text-[hsl(var(--text-secondary))]">
          Model placement comes from the selected runtime target.
        </div>
      )}
      {(controls.showGpuLayers || controls.showTensorSplit || controls.showContextSize) && (
        <AdvancedPlacementControls
          controls={controls}
          formState={formState}
          setContextSize={setContextSize}
          setGpuLayers={setGpuLayers}
          setTensorSplit={setTensorSplit}
        />
      )}
    </>
  );
}

function DeviceControls({
  formState,
  setDeviceId,
  setDeviceMode,
  showDeviceId,
}: {
  formState: ModelServeFormState;
  setDeviceId: (value: string) => void;
  setDeviceMode: (value: RuntimeDeviceMode) => void;
  showDeviceId: boolean;
}) {
  return (
    <div className="grid grid-cols-1 gap-3 md:grid-cols-2">
      <label className="grid gap-1 text-xs text-[hsl(var(--text-secondary))]">
        Model device
        <select
          value={formState.deviceMode}
          onChange={(event) => setDeviceMode(event.target.value as RuntimeDeviceMode)}
          className="rounded border border-[hsl(var(--border-default))] bg-[hsl(var(--surface-base))] px-2 py-1.5 text-sm text-[hsl(var(--text-primary))]"
        >
          {DEVICE_OPTIONS.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      </label>
      {showDeviceId && (
        <label className="grid gap-1 text-xs text-[hsl(var(--text-secondary))]">
          Device ID
          <input
            value={formState.deviceId}
            onChange={(event) => setDeviceId(event.target.value)}
            className="rounded border border-[hsl(var(--border-default))] bg-[hsl(var(--surface-base))] px-2 py-1.5 text-sm text-[hsl(var(--text-primary))]"
          />
        </label>
      )}
    </div>
  );
}

function AdvancedPlacementControls({
  controls,
  formState,
  setContextSize,
  setGpuLayers,
  setTensorSplit,
}: {
  controls: ModelServeControls;
  formState: ModelServeFormState;
  setContextSize: (value: string) => void;
  setGpuLayers: (value: string) => void;
  setTensorSplit: (value: string) => void;
}) {
  return (
    <div className="grid grid-cols-1 gap-3 md:grid-cols-3">
      {controls.showGpuLayers && (
        <label className="grid gap-1 text-xs text-[hsl(var(--text-secondary))]">
          Model GPU layers
          <input
            type="number"
            value={formState.gpuLayers}
            onChange={(event) => setGpuLayers(event.target.value)}
            className="rounded border border-[hsl(var(--border-default))] bg-[hsl(var(--surface-base))] px-2 py-1.5 text-sm text-[hsl(var(--text-primary))]"
          />
        </label>
      )}
      {controls.showTensorSplit && (
        <label className="grid gap-1 text-xs text-[hsl(var(--text-secondary))]">
          Model tensor split
          <input
            value={formState.tensorSplit}
            onChange={(event) => setTensorSplit(event.target.value)}
            className="rounded border border-[hsl(var(--border-default))] bg-[hsl(var(--surface-base))] px-2 py-1.5 text-sm text-[hsl(var(--text-primary))]"
          />
        </label>
      )}
      {controls.showContextSize && (
        <label className="grid gap-1 text-xs text-[hsl(var(--text-secondary))]">
          Context
          <input
            type="number"
            value={formState.contextSize}
            onChange={(event) => setContextSize(event.target.value)}
            className="rounded border border-[hsl(var(--border-default))] bg-[hsl(var(--surface-base))] px-2 py-1.5 text-sm text-[hsl(var(--text-primary))]"
          />
        </label>
      )}
    </div>
  );
}
