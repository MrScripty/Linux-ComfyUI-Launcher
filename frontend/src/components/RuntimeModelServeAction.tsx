import { Play, Square } from 'lucide-react';
import type { ModelInfo } from '../types/apps';
import type { ServedModelStatus } from '../types/api-serving';
import type { LocalModelRowState } from './LocalModelRowState';
import { IconButton } from './ui';

export interface RuntimeModelServeActionProps {
  model: ModelInfo;
  rowState: LocalModelRowState;
  servedStatus?: ServedModelStatus | null;
  onServeModel?: (model: ModelInfo) => void;
}

export function RuntimeModelServeAction({
  model,
  rowState,
  servedStatus,
  onServeModel,
}: RuntimeModelServeActionProps) {
  if (!onServeModel) {
    return null;
  }

  return (
    <IconButton
      icon={servedStatus ? <Square /> : <Play />}
      tooltip={servedStatus ? 'Unload model' : 'Serve model'}
      onClick={() => onServeModel(model)}
      disabled={rowState.isPartialDownload}
      size="sm"
      active={Boolean(servedStatus)}
      className={
        servedStatus
          ? 'text-[hsl(var(--accent-success))] bg-[hsl(var(--accent-success)/0.12)]'
          : undefined
      }
    />
  );
}
