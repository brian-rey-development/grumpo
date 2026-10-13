import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect, it, vi } from 'vitest';

import { AppShell } from '@/layout/AppShell';
import { useNavigation } from '@/layout/navigation';
import { useToolbarTitle } from '@/layout/toolbarTitle';
import { renderWithProviders } from '@/test/render';

function Screen({ title }: { title: string }) {
  const { screen: current } = useNavigation();
  useToolbarTitle(current === 'today' ? title : null);
  return <p>{`${current} content`}</p>;
}

function renderShell(title: string) {
  return renderWithProviders(
    <AppShell>
      <Screen title={title} />
    </AppShell>,
  );
}

it('replaces the toolbar title while the screen is shown', () => {
  renderShell('Yesterday');

  expect(screen.getByRole('heading', { level: 1 }).textContent).toBe('Yesterday');
});

it('follows the title as it changes', () => {
  const { rerender } = renderShell('Yesterday');

  rerender(
    <AppShell>
      <Screen title="Mon 5 Oct" />
    </AppShell>,
  );

  expect(screen.getByRole('heading', { level: 1 }).textContent).toBe('Mon 5 Oct');
});

it('restores the default title once the screen is gone', async () => {
  const user = userEvent.setup();
  renderShell('Yesterday');

  await user.click(screen.getByRole('button', { name: 'Rules' }));

  await waitFor(() => {
    expect(screen.getByRole('heading', { level: 1 }).textContent).toBe('Rules');
  });
});

it('refuses to run outside the shell', () => {
  const silence = vi.spyOn(console, 'error').mockImplementation(() => undefined);

  expect(() => renderWithProviders(<Screen title="Yesterday" />)).toThrow(/AppShell/);
  silence.mockRestore();
});
