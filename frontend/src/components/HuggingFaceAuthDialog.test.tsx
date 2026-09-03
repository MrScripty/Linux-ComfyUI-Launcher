import { useState } from 'react';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { HuggingFaceAuthDialog } from './HuggingFaceAuthDialog';

const { getAuthStatusMock } = vi.hoisted(() => ({
  getAuthStatusMock: vi.fn(),
}));

vi.mock('../api/adapter', () => ({
  api: {
    get_hf_auth_status: getAuthStatusMock,
  },
  isAPIAvailable: () => true,
}));

function AuthDialogHarness() {
  const [isOpen, setIsOpen] = useState(false);
  return (
    <>
      <button type="button" onClick={() => setIsOpen(true)}>Open authentication</button>
      <HuggingFaceAuthDialog isOpen={isOpen} onClose={() => setIsOpen(false)} />
    </>
  );
}

describe('HuggingFaceAuthDialog', () => {
  beforeEach(() => {
    getAuthStatusMock.mockReset();
    getAuthStatusMock.mockResolvedValue({
      authenticated: false,
      success: true,
    });
  });

  it('names the modal, focuses its token field, and restores its opener on Escape', async () => {
    render(<AuthDialogHarness />);
    const trigger = screen.getByRole('button', { name: 'Open authentication' });
    trigger.focus();
    fireEvent.click(trigger);

    expect(
      await screen.findByRole('dialog', { name: 'HuggingFace Authentication' })
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: 'Close HuggingFace authentication' })
    ).toBeInTheDocument();
    await waitFor(() => expect(screen.getByLabelText('Access Token')).toHaveFocus());

    fireEvent.keyDown(document, { key: 'Escape' });
    await waitFor(() => {
      expect(screen.queryByRole('dialog', { name: 'HuggingFace Authentication' }))
        .not.toBeInTheDocument();
    });
    expect(trigger).toHaveFocus();
  });

  it('retains backdrop dismissal through the modal Interface', async () => {
    render(<AuthDialogHarness />);
    fireEvent.click(screen.getByRole('button', { name: 'Open authentication' }));
    const dialog = await screen.findByRole('dialog', { name: 'HuggingFace Authentication' });
    const backdrop = dialog.parentElement?.querySelector<HTMLElement>('[data-modal-backdrop]');
    if (!backdrop) {
      throw new TypeError('Expected authentication dialog backdrop');
    }

    fireEvent.mouseDown(backdrop);
    await waitFor(() => {
      expect(screen.queryByRole('dialog', { name: 'HuggingFace Authentication' }))
        .not.toBeInTheDocument();
    });
  });
});
