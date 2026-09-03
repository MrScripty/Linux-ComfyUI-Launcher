import { useState } from 'react';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type { RemoteModelInfo } from '../types/apps';
import { RemoteModelListItemActions } from './RemoteModelListItemActions';
import type { RemoteDownloadFlags } from './RemoteModelListItemState';

const model: RemoteModelInfo = {
  repoId: 'org/test-model',
  name: 'Test Model',
  developer: 'org',
  kind: 'text-generation',
  formats: ['gguf'],
  quants: ['Q4_K_M'],
  downloadOptions: [
    {
      quant: 'Q4_K_M',
      sizeBytes: 2 * 1024 ** 3,
      fileGroup: null,
    },
  ],
  url: 'https://huggingface.co/org/test-model',
  totalSizeBytes: 2 * 1024 ** 3,
};

const idleFlags: RemoteDownloadFlags = {
  isDownloading: false,
  isErrored: false,
  isPaused: false,
  isPausing: false,
  isQueued: false,
};

interface ActionsHarnessProps {
  flags?: RemoteDownloadFlags;
  hasExactDetails?: boolean;
  onCancelDownload?: (downloadKey: string) => Promise<void>;
  onHydrateModelDetails?: (modelToHydrate: RemoteModelInfo) => Promise<void>;
}

function ActionsHarness({
  flags = idleFlags,
  hasExactDetails = true,
  onCancelDownload = vi.fn().mockResolvedValue(undefined),
  onHydrateModelDetails,
}: ActionsHarnessProps) {
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  const downloadOptions = model.downloadOptions ?? [];

  return (
    <RemoteModelListItemActions
      downloadOptions={downloadOptions}
      flags={flags}
      hasExactDetails={hasExactDetails}
      hasFileGroups={false}
      isHydratingDetails={false}
      isMenuOpen={isMenuOpen}
      model={model}
      downloadKey="org/test-model"
      progressDegrees={120}
      selectedGroups={new Set()}
      selectedTotalBytes={0}
      onCancelDownload={onCancelDownload}
      onClearSelection={vi.fn()}
      onCloseMenu={() => setIsMenuOpen(false)}
      onHydrateModelDetails={onHydrateModelDetails}
      onOpenUrl={vi.fn()}
      onPauseDownload={vi.fn().mockResolvedValue(undefined)}
      onResumeDownload={vi.fn().mockResolvedValue(undefined)}
      onStartDownload={vi.fn().mockResolvedValue(undefined)}
      onToggleGroup={vi.fn()}
      onToggleMenu={() => setIsMenuOpen((current) => !current)}
    />
  );
}

describe('RemoteModelListItemActions', () => {
  it('owns the download popup relationship, focus entry, Escape dismissal, and return', async () => {
    render(<ActionsHarness />);
    const trigger = screen.getByRole('button', { name: 'Download options' });
    expect(trigger).toHaveAttribute('aria-expanded', 'false');
    expect(trigger).toHaveAttribute('aria-haspopup', 'dialog');

    trigger.focus();
    fireEvent.click(trigger);
    const panel = screen.getByRole('dialog', { name: 'Download options for Test Model' });
    expect(trigger).toHaveAttribute('aria-controls', panel.id);
    await waitFor(() =>
      expect(screen.getByRole('button', { name: /Q4_K_M/ })).toHaveFocus()
    );

    fireEvent.keyDown(document, { key: 'Escape' });
    await waitFor(() =>
      expect(
        screen.queryByRole('dialog', { name: 'Download options for Test Model' })
      ).not.toBeInTheDocument()
    );
    expect(trigger).toHaveFocus();
  });

  it('uses queue-another as the popup opener while download cancellation stays direct', async () => {
    const onCancelDownload = vi.fn().mockResolvedValue(undefined);
    render(
      <ActionsHarness
        flags={{ ...idleFlags, isDownloading: true }}
        onCancelDownload={onCancelDownload}
      />
    );
    const queueTrigger = screen.getByRole('button', { name: 'Queue another download' });
    const cancelButton = screen.getByRole('button', { name: 'Cancel download' });
    expect(queueTrigger).toHaveAttribute('aria-haspopup', 'dialog');
    expect(cancelButton).not.toHaveAttribute('aria-haspopup');

    queueTrigger.focus();
    fireEvent.click(queueTrigger);
    await waitFor(() =>
      expect(screen.getByRole('button', { name: /Q4_K_M/ })).toHaveFocus()
    );
    fireEvent.keyDown(document, { key: 'Escape' });
    await waitFor(() => expect(queueTrigger).toHaveFocus());

    fireEvent.click(cancelButton);
    expect(onCancelDownload).toHaveBeenCalledWith('org/test-model');
  });

  it('hydrates incomplete details only when opening download options', () => {
    const onHydrateModelDetails = vi.fn().mockResolvedValue(undefined);
    render(
      <ActionsHarness
        hasExactDetails={false}
        onHydrateModelDetails={onHydrateModelDetails}
      />
    );
    const trigger = screen.getByRole('button', { name: 'Download options' });

    fireEvent.click(trigger);
    expect(onHydrateModelDetails).toHaveBeenCalledWith(model);

    fireEvent.click(trigger);
    expect(onHydrateModelDetails).toHaveBeenCalledTimes(1);
  });
});
