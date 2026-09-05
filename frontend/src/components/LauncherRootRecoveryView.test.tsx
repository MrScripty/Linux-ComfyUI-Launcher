import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { LauncherRootRecoveryView } from './LauncherRootRecoveryView';

describe('LauncherRootRecoveryView', () => {
  it('names the recovery region, announces status, focuses its heading, and keeps window controls available', () => {
    const onClose = vi.fn();
    const onMinimize = vi.fn();

    render(
      <LauncherRootRecoveryView
        title="Desktop bridge unavailable"
        message="Close and reopen Pumas Library."
        onClose={onClose}
        onMinimize={onMinimize}
      />
    );

    const heading = screen.getByRole('heading', { name: 'Desktop bridge unavailable' });
    expect(screen.getByRole('region', { name: 'Desktop bridge unavailable' })).toBeVisible();
    expect(screen.getByRole('status')).toHaveTextContent('Close and reopen Pumas Library.');
    expect(screen.getByRole('status')).toHaveAttribute('aria-live', 'polite');
    expect(screen.getByRole('status')).toHaveAttribute('aria-atomic', 'true');
    expect(heading).toHaveFocus();

    fireEvent.click(screen.getByRole('button', { name: 'Minimize' }));
    fireEvent.click(screen.getByRole('button', { name: 'Close' }));
    expect(onMinimize).toHaveBeenCalledTimes(1);
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('exposes named primary and secondary recovery actions', () => {
    const onPrimary = vi.fn();
    const onSecondary = vi.fn();

    render(
      <LauncherRootRecoveryView
        title="Library needs attention"
        message="Select an existing library to continue."
        primaryAction={{ label: 'Select Library', onAction: onPrimary }}
        secondaryAction={{ label: 'Back to Library', onAction: onSecondary }}
        onClose={vi.fn()}
        onMinimize={vi.fn()}
      />
    );

    fireEvent.click(screen.getByRole('button', { name: 'Select Library' }));
    fireEvent.click(screen.getByRole('button', { name: 'Back to Library' }));
    expect(onPrimary).toHaveBeenCalledTimes(1);
    expect(onSecondary).toHaveBeenCalledTimes(1);
  });

  it('hides unsupported minimize while retaining a working close action', () => {
    const onClose = vi.fn();

    render(
      <LauncherRootRecoveryView
        title="Desktop bridge unavailable"
        message="Close and reopen Pumas Library."
        onClose={onClose}
      />
    );

    expect(screen.queryByRole('button', { name: 'Minimize' })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Close' }));
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
