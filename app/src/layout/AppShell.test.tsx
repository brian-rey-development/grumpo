import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect, it } from 'vitest';

import { AppShell } from '@/layout/AppShell';
import { SCREENS, useNavigation } from '@/layout/navigation';
import { ToolbarActions } from '@/layout/Toolbar';
import { expectAccessible, renderWithProviders } from '@/test/render';

function Screen() {
  const { screen: current } = useNavigation();
  return (
    <>
      <p data-testid="content">{`${current} content`}</p>
      {current === 'today' ? (
        <ToolbarActions>
          <button type="button">Previous day</button>
        </ToolbarActions>
      ) : null}
    </>
  );
}

function renderShell() {
  return renderWithProviders(
    <AppShell sidebarFooter={<span>Footer</span>}>
      <Screen />
    </AppShell>,
  );
}

it('shows the screen title at the head of the page and keeps the toolbar as the drag region', () => {
  const { container } = renderShell();

  const title = screen.getByRole('heading', { level: 1, name: 'Today' });
  expect(title.closest('[data-screen]')).not.toBeNull();
  expect(title.closest('header')).toBeNull();
  expect(container.querySelector('header')?.hasAttribute('data-tauri-drag-region')).toBe(true);
});

it('keeps the drag region off the controls so they stay clickable', () => {
  const { container } = renderShell();

  const button = screen.getByRole('button', { name: 'Previous day' });

  expect(button.closest('[data-tauri-drag-region]')).toBe(container.querySelector('header'));
  expect(button.hasAttribute('data-tauri-drag-region')).toBe(false);
});

it('renders per screen actions in the toolbar and removes them when the screen changes', async () => {
  const user = userEvent.setup();
  const { container } = renderShell();
  const banner = container.querySelector('header');
  if (banner === null) throw new Error('Missing toolbar');
  expect(within(banner).getByRole('button', { name: 'Previous day' })).toBeTruthy();

  await user.click(screen.getByRole('button', { name: 'Week' }));

  await waitFor(() => {
    expect(screen.queryByRole('button', { name: 'Previous day' })).toBeNull();
  });
});

it('names the sidebar Grumpo and keeps its title bar row empty for the window controls', () => {
  renderShell();
  const sidebar = screen.getByRole('complementary', { name: 'Grumpo' });
  const titlebar = sidebar.querySelector('[data-tauri-drag-region]');
  const brand = within(sidebar).getByText('Grumpo');

  expect(titlebar?.textContent).toBe('');
  expect(titlebar?.contains(brand)).toBe(false);
  expect(brand.parentElement?.querySelector('img')?.getAttribute('alt')).toBe('');
  expect(within(sidebar).getByText('Footer')).toBeTruthy();
});

it('shows at most one mascot in the whole shell', () => {
  const { container } = renderShell();

  expect(container.querySelectorAll('img')).toHaveLength(1);
});

it('marks a screen with unsaved work using a dot that is also spoken', () => {
  renderWithProviders(
    <AppShell unsavedScreens={['rules']}>
      <Screen />
    </AppShell>,
  );

  const rules = screen.getByRole('button', { name: 'Rules, unsaved changes' });
  expect(rules.querySelector('[data-testid="unsaved-dot"]')?.getAttribute('aria-hidden')).toBe(
    'true',
  );
  expect(screen.getAllByTestId('unsaved-dot')).toHaveLength(1);
  expect(screen.getByRole('button', { name: 'Today' })).toBeTruthy();
});

it('shows no dots when no screen has unsaved work', () => {
  renderShell();

  expect(screen.queryByTestId('unsaved-dot')).toBeNull();
});

it('crossfades screens in parallel, each in its own scroll container', async () => {
  const user = userEvent.setup();
  const { container } = renderShell();
  const today = container.querySelector('[data-screen="today"]');

  await user.click(screen.getByRole('button', { name: 'Rules' }));

  expect(container.querySelectorAll('[data-screen]')).toHaveLength(2);
  await waitFor(() => {
    expect(container.querySelectorAll('[data-screen]')).toHaveLength(1);
  });
  const rules = container.querySelector('[data-screen="rules"]');
  expect(rules).not.toBeNull();
  expect(rules).not.toBe(today);
  expect(SCREENS).toContain('rules');
});

it('has no accessibility violations', async () => {
  const { container } = renderShell();

  await expectAccessible(container, true);
});
