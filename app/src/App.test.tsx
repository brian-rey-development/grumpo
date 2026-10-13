import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, expect, it, vi } from 'vitest';

const { subscribeMock, invokeMock } = vi.hoisted(() => ({
  subscribeMock:
    vi.fn<(event: string, handler: (payload: unknown) => void) => Promise<() => void>>(),
  invokeMock: vi.fn<(command: string, ...args: unknown[]) => Promise<unknown>>(),
}));

vi.mock('@/ipc/events', () => ({ subscribe: subscribeMock }));
vi.mock('@/ipc/commands', () => ({ invokeCommand: invokeMock }));

import { App } from '@/App';
import { routeCommands } from '@/test/ipc';
import { createTestQueryClient, expectAccessible } from '@/test/render';

beforeEach(() => {
  subscribeMock.mockReset();
  subscribeMock.mockResolvedValue(vi.fn());
  routeCommands(invokeMock);
});

function renderApp() {
  return render(<App client={createTestQueryClient()} />);
}

async function renderOpenApp() {
  const view = renderApp();
  await screen.findByRole('navigation', { name: 'Main' });
  return view;
}

it('renders the sidebar with Today selected and the screen title in the toolbar', async () => {
  await renderOpenApp();

  const nav = screen.getByRole('navigation', { name: 'Main' });
  expect(screen.getAllByRole('button').filter((button) => nav.contains(button))).toHaveLength(4);
  expect(screen.getByRole('complementary', { name: 'Grumpo' })).toBeTruthy();
  expect(screen.getByRole('button', { name: 'Today' }).getAttribute('aria-current')).toBe('page');
  expect(screen.getByRole('heading', { level: 1, name: 'Today' })).toBeTruthy();
  expect(screen.getAllByRole('heading', { level: 1 })).toHaveLength(1);
});

it('switches screens from the sidebar and with Cmd+number, announcing each change politely', async () => {
  const user = userEvent.setup();
  await renderOpenApp();
  const announcer = screen.getByRole('status', { name: 'Current screen' });
  expect(announcer.textContent).toBe('');

  await user.click(screen.getByRole('button', { name: 'Rules' }));
  await waitFor(() => {
    expect(screen.getByRole('heading', { level: 1, name: 'Rules' })).toBeTruthy();
  });
  expect(screen.getByRole('button', { name: 'Rules' }).getAttribute('aria-current')).toBe('page');
  expect(screen.getByRole('button', { name: 'Today' }).getAttribute('aria-current')).toBeNull();
  expect(announcer.textContent).toBe('Rules');

  fireEvent.keyDown(window, { key: '2', metaKey: true });
  await waitFor(() => {
    expect(screen.getByRole('heading', { level: 1, name: 'Last 7 days' })).toBeTruthy();
  });
  expect(announcer.textContent).toBe('Week');
});

it('keeps unsaved rule edits across screens, marking Rules in the sidebar until they are saved', async () => {
  const user = userEvent.setup();
  await renderOpenApp();
  await user.click(screen.getByRole('button', { name: 'Rules' }));
  await user.click(await screen.findByRole('button', { name: 'Remove reddit.com' }));
  expect(screen.getByTestId('unsaved-dot')).toBeTruthy();
  expect(screen.getByRole('button', { name: 'Rules, unsaved changes' })).toBeTruthy();

  await user.click(screen.getByRole('button', { name: 'Today' }));
  await waitFor(() => {
    expect(screen.getByRole('heading', { level: 1, name: 'Today' })).toBeTruthy();
  });
  expect(screen.getByTestId('unsaved-dot')).toBeTruthy();

  await user.click(screen.getByRole('button', { name: /^Rules/ }));
  expect(await screen.findByRole('button', { name: 'Save changes' })).toBeTruthy();
  expect(screen.queryByText('reddit.com')).toBeNull();

  await user.click(screen.getByRole('button', { name: 'Discard' }));
  expect(await screen.findByText('reddit.com')).toBeTruthy();
  expect(screen.queryByTestId('unsaved-dot')).toBeNull();
  expect(screen.getByRole('button', { name: 'Rules' })).toBeTruthy();
});

it('shows no dot on Rules when nothing is waiting to be saved', async () => {
  const user = userEvent.setup();
  await renderOpenApp();
  await user.click(screen.getByRole('button', { name: 'Rules' }));
  await screen.findByText('reddit.com');

  expect(screen.queryByTestId('unsaved-dot')).toBeNull();
});

it('subscribes to backend events once mounted', async () => {
  renderApp();

  await waitFor(() => {
    expect(subscribeMock).toHaveBeenCalledWith('usage-changed', expect.any(Function));
  });
  expect(subscribeMock).toHaveBeenCalledWith('status-changed', expect.any(Function));
});

it('has no accessibility violations, landmarks included', async () => {
  const { container } = renderApp();
  await act(async () => {
    await Promise.resolve();
  });

  await expectAccessible(container, true);
});
