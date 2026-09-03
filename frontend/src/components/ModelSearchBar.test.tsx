import { useState } from 'react';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ModelSearchBar } from './ModelSearchBar';

function ModelSearchBarHarness() {
  const [isOpen, setIsOpen] = useState(false);
  const [selectedFilter, setSelectedFilter] = useState('text-generation');

  return (
    <ModelSearchBar
      searchQuery=""
      onSearchChange={vi.fn()}
      isDownloadMode={false}
      onToggleMode={vi.fn()}
      isCategoryFiltered
      onFilterClick={() => setIsOpen((current) => !current)}
      totalModels={3}
      showCategoryMenu={isOpen}
      filterList={['all', 'text-generation', 'image-generation']}
      selectedFilter={selectedFilter}
      onSelectFilter={(filter) => {
        setSelectedFilter(filter);
        setIsOpen(false);
      }}
    />
  );
}

describe('ModelSearchBar', () => {
  it('relates the filter trigger to its popup and restores focus after dismissal', async () => {
    render(<ModelSearchBarHarness />);
    const trigger = screen.getByRole('button', { name: 'Filter by category' });
    expect(trigger).toHaveAttribute('aria-expanded', 'false');
    expect(trigger).toHaveAttribute('aria-haspopup', 'dialog');

    trigger.focus();
    fireEvent.click(trigger);
    const panel = screen.getByRole('dialog', { name: 'Filter by category' });
    expect(trigger).toHaveAttribute('aria-controls', panel.id);
    expect(trigger).toHaveAttribute('aria-expanded', 'true');
    await waitFor(() =>
      expect(screen.getByRole('button', { name: 'text-generation' })).toHaveFocus()
    );

    fireEvent.keyDown(document, { key: 'Escape' });
    await waitFor(() =>
      expect(screen.queryByRole('dialog', { name: 'Filter by category' })).not.toBeInTheDocument()
    );
    expect(trigger).toHaveFocus();
  });

  it('preserves selection behavior while closing the popup', async () => {
    render(<ModelSearchBarHarness />);
    const trigger = screen.getByRole('button', { name: 'Filter by category' });
    trigger.focus();
    fireEvent.click(trigger);
    fireEvent.click(await screen.findByRole('button', { name: 'image-generation' }));

    await waitFor(() =>
      expect(screen.queryByRole('dialog', { name: 'Filter by category' })).not.toBeInTheDocument()
    );
    expect(screen.getByRole('button', { name: 'Filter by category' })).toHaveFocus();
  });
});
