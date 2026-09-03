import { useState } from 'react';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ConfirmationDialog } from './ConfirmationDialog';
import { InstallDialogFrame } from './InstallDialogFrame';

function NestedInstallDialogHarness() {
  const [isInstallOpen, setIsInstallOpen] = useState(false);
  const [isConfirmationOpen, setIsConfirmationOpen] = useState(false);
  return (
    <>
      <button type="button" onClick={() => setIsInstallOpen(true)}>Open install</button>
      <InstallDialogFrame
        isOpen={isInstallOpen}
        isPageMode={false}
        onClose={() => setIsInstallOpen(false)}
        title="Install Runtime Version"
      >
        <button type="button" onClick={() => setIsConfirmationOpen(true)}>Cancel installation</button>
        <ConfirmationDialog
          confirmLabel="Confirm cancellation"
          isOpen={isConfirmationOpen}
          message="Stop the active installation."
          onCancel={() => setIsConfirmationOpen(false)}
          onConfirm={() => setIsConfirmationOpen(false)}
          title="Cancel installation?"
        />
      </InstallDialogFrame>
    </>
  );
}

describe('InstallDialogFrame', () => {
  it('renders modal mode as a named dialog and closes from backdrop or Escape key', () => {
    const onClose = vi.fn();

    render(
      <InstallDialogFrame
        isOpen={true}
        isPageMode={false}
        onClose={onClose}
        title="Install Runtime Version"
      >
        <div>Install content</div>
      </InstallDialogFrame>
    );

    expect(screen.getByRole('dialog', { name: 'Install Runtime Version' })).toBeInTheDocument();

    const dialog = screen.getByRole('dialog', { name: 'Install Runtime Version' });
    const backdrop = dialog.parentElement?.querySelector<HTMLElement>('[data-modal-backdrop]');
    if (!backdrop) {
      throw new TypeError('Expected install dialog backdrop');
    }
    fireEvent.mouseDown(backdrop);
    expect(onClose).toHaveBeenCalledTimes(1);

    fireEvent.keyDown(document, { key: 'Escape' });
    expect(onClose).toHaveBeenCalledTimes(2);
  });

  it('renders page mode without modal dialog chrome', () => {
    render(
      <InstallDialogFrame
        isOpen={true}
        isPageMode={true}
        onClose={vi.fn()}
        title="Install Runtime Version"
      >
        <div>Install content</div>
      </InstallDialogFrame>
    );

    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    expect(document.querySelector('[data-modal-backdrop]')).not.toBeInTheDocument();
    expect(screen.getByText('Install content')).toBeInTheDocument();
  });

  it('keeps nested confirmation Escape and restoration ordered ahead of its parent', async () => {
    render(<NestedInstallDialogHarness />);
    const outerTrigger = screen.getByRole('button', { name: 'Open install' });
    outerTrigger.focus();
    fireEvent.click(outerTrigger);
    const innerTrigger = screen.getByRole('button', { name: 'Cancel installation' });
    innerTrigger.focus();
    fireEvent.click(innerTrigger);

    await waitFor(() => expect(screen.getByRole('button', { name: 'Cancel' })).toHaveFocus());
    fireEvent.keyDown(document, { key: 'Escape' });
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog', { name: 'Cancel installation?' }))
        .not.toBeInTheDocument();
    });
    expect(screen.getByRole('dialog', { name: 'Install Runtime Version' })).toBeInTheDocument();
    expect(innerTrigger).toHaveFocus();

    fireEvent.keyDown(document, { key: 'Escape' });
    await waitFor(() => {
      expect(screen.queryByRole('dialog', { name: 'Install Runtime Version' }))
        .not.toBeInTheDocument();
    });
    expect(outerTrigger).toHaveFocus();
  });
});
