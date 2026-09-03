import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ModelImportDialog } from './ModelImportDialog';

describe('ModelImportDialog', () => {
  it('uses a named modal lifecycle while preserving explicit-only backdrop dismissal', async () => {
    const onClose = vi.fn();
    render(
      <ModelImportDialog
        importPaths={[]}
        onClose={onClose}
        onImportComplete={vi.fn()}
      />
    );

    const dialog = await screen.findByRole('dialog', { name: 'Import Models' });
    const closeButton = screen.getByRole('button', { name: 'Close model import dialog' });
    await waitFor(() => expect(closeButton).toHaveFocus());

    const backdrop = dialog.parentElement?.querySelector<HTMLElement>('[data-modal-backdrop]');
    if (!backdrop) {
      throw new TypeError('Expected model import dialog backdrop');
    }
    fireEvent.mouseDown(backdrop);
    expect(onClose).not.toHaveBeenCalled();

    fireEvent.keyDown(document, { key: 'Escape' });
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
