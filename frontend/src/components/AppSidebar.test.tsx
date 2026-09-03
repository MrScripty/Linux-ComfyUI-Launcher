import { fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { Box } from 'lucide-react';
import type { AppConfig } from '../types/apps';
import { AppSidebar } from './AppSidebar';

const mockApps: AppConfig[] = [
  {
    id: 'ollama',
    name: 'ollama',
    displayName: 'Ollama',
    icon: Box,
    status: 'running',
    iconState: 'running',
  },
  {
    id: 'llama-cpp',
    name: 'llama-cpp',
    displayName: 'llama.cpp',
    icon: Box,
    status: 'idle',
    iconState: 'offline',
  },
];

describe('AppSidebar', () => {
  beforeEach(() => vi.clearAllMocks());

  it('renders only the supplied compiled plugin icons', () => {
    render(<AppSidebar apps={mockApps} selectedAppId={null} onSelectApp={vi.fn()} />);

    expect(screen.getByRole('toolbar', { name: 'Inference plugins' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Ollama' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'llama.cpp' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Add app' })).not.toBeInTheDocument();
  });

  it('selects a plugin and returns home on Escape', async () => {
    const user = userEvent.setup();
    const onSelectApp = vi.fn();
    const { rerender } = render(
      <AppSidebar apps={mockApps} selectedAppId={null} onSelectApp={onSelectApp} />
    );

    await user.click(screen.getByRole('button', { name: 'Ollama' }));
    expect(onSelectApp).toHaveBeenCalledWith('ollama');

    rerender(<AppSidebar apps={mockApps} selectedAppId="ollama" onSelectApp={onSelectApp} />);
    fireEvent.keyDown(window, { key: 'Escape' });
    expect(onSelectApp).toHaveBeenCalledWith(null);
  });
});
