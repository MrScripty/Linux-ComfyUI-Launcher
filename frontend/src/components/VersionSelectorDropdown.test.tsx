import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { VersionSelectorDropdown } from './VersionSelectorDropdown';

function renderDropdown(
  overrides: Partial<React.ComponentProps<typeof VersionSelectorDropdown>> = {}
) {
  const props: React.ComponentProps<typeof VersionSelectorDropdown> = {
    activeVersion: 'v1.0.0',
    combinedVersions: ['v1.0.0', 'v1.1.0'],
    defaultVersion: 'v1.0.0',
    hasVersionsToShow: true,
    installedVersions: ['v1.0.0', 'v1.1.0'],
    installingVersion: null,
    isInstallComplete: false,
    isLoading: false,
    isOpen: true,
    isSwitching: false,
    onMakeDefault: vi.fn().mockResolvedValue(true),
    onSwitchVersion: vi.fn(),
    ...overrides,
  };

  render(<VersionSelectorDropdown {...props} />);
  return props;
}

describe('VersionSelectorDropdown', () => {
  it('uses native controls for switching and defaults', async () => {
    const props = renderDropdown();

    fireEvent.click(screen.getByRole('button', { name: 'Switch to v1.1.0' }));
    expect(props.onSwitchVersion).toHaveBeenCalledWith('v1.1.0');

    fireEvent.click(screen.getByRole('button', { name: 'Set v1.1.0 as default' }));
    expect(props.onMakeDefault).toHaveBeenCalledWith('v1.1.0');
    expect(props.onSwitchVersion).toHaveBeenCalledTimes(1);

    expect(screen.queryByText(/shortcut/i)).not.toBeInTheDocument();
  });
});
