import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn<(command: string, ...args: unknown[]) => Promise<unknown>>(),
}));

vi.mock('@/ipc/commands', () => ({ invokeCommand: invokeMock }));

import { AppsEditor } from '@/features/rules/AppsEditor';
import { RULE_CAPS } from '@/features/rules/rulesModel';
import { CommandFailure } from '@/ipc/errors';
import type { Rules } from '@/ipc/generated/Rules';
import { EMPTY_RULES, SAFARI, SAMPLE_RULES, SLACK } from '@/test/ipc';
import { expectAccessible } from '@/test/render';
import { installRulesCommands, renderEditor } from '@/test/rules';

beforeEach(() => {
  invokeMock.mockReset();
  installRulesCommands(invokeMock);
});

function renderApps(initial: Rules = SAMPLE_RULES, readOnly = false) {
  return renderEditor((props) => <AppsEditor {...props} />, { initial, readOnly });
}

function optionNames(): (string | null)[] {
  return screen
    .getAllByRole('option')
    .map((option) => option.querySelectorAll('span')[1]?.textContent ?? null);
}

it('lists blocked apps with their names and a remove button', () => {
  renderApps();

  expect(screen.getByText('Slack')).toBeTruthy();
  expect(screen.getByRole('button', { name: 'Remove Slack' })).toBeTruthy();
});

it('explains an empty list and still offers to add', () => {
  renderApps(EMPTY_RULES);

  expect(screen.getByRole('heading', { name: 'No blocked apps' })).toBeTruthy();
  expect(screen.getByRole('button', { name: 'Add app' })).toBeTruthy();
});

it('removes an app', async () => {
  const user = userEvent.setup();
  const rendered = renderApps();

  await user.click(screen.getByRole('button', { name: 'Remove Slack' }));

  expect(rendered.currentRules().blockedApps).toEqual([]);
});

it('opens a picker with running apps first and hides apps already blocked', async () => {
  const user = userEvent.setup();
  renderApps();

  await user.click(screen.getByRole('button', { name: 'Add app' }));

  const dialog = await screen.findByRole('dialog', { name: 'Add an app' });
  await within(dialog).findAllByRole('option');
  expect(optionNames()).toEqual(['Safari', 'Mail', 'Notes']);
  expect(within(dialog).getByText('Running now')).toBeTruthy();
  expect(within(dialog).queryByRole('option', { name: /Slack/ })).toBeNull();
});

it('filters by name as you type', async () => {
  const user = userEvent.setup();
  renderApps(EMPTY_RULES);
  await user.click(screen.getByRole('button', { name: 'Add app' }));
  await screen.findAllByRole('option');

  await user.type(screen.getByRole('combobox', { name: 'Search apps' }), 'SLA');

  expect(optionNames()).toEqual(['Slack']);
});

it('says when nothing matches the search', async () => {
  const user = userEvent.setup();
  renderApps(EMPTY_RULES);
  await user.click(screen.getByRole('button', { name: 'Add app' }));
  await screen.findAllByRole('option');

  await user.type(screen.getByRole('combobox', { name: 'Search apps' }), 'zzz');

  expect(screen.queryAllByRole('option')).toHaveLength(0);
  expect(screen.getByText('No app matches "zzz".')).toBeTruthy();
});

it('is driven from the keyboard with the arrow keys and Enter', async () => {
  const user = userEvent.setup();
  const rendered = renderApps(EMPTY_RULES);
  await user.click(screen.getByRole('button', { name: 'Add app' }));
  await screen.findAllByRole('option');
  const search = screen.getByRole('combobox', { name: 'Search apps' });

  const options = screen.getAllByRole('option');
  expect(search.getAttribute('aria-activedescendant')).toBe(options[0]?.id);
  expect(options[0]?.getAttribute('aria-selected')).toBe('true');
  expect(document.activeElement).toBe(search);
  await user.keyboard('{ArrowDown}{ArrowDown}');
  expect(search.getAttribute('aria-activedescendant')).toBe(options[2]?.id);
  await user.keyboard('{ArrowDown}{ArrowDown}{ArrowDown}');
  expect(search.getAttribute('aria-activedescendant')).toBe(options[3]?.id);
  await user.keyboard('{ArrowUp}{Home}{End}{Home}{Enter}');

  expect(rendered.currentRules().blockedApps).toEqual([SAFARI]);
  await waitFor(() => {
    expect(screen.queryByRole('dialog')).toBeNull();
  });
});

it('adds the app that is clicked and closes the picker', async () => {
  const user = userEvent.setup();
  const rendered = renderApps(EMPTY_RULES);
  await user.click(screen.getByRole('button', { name: 'Add app' }));

  await user.click(await screen.findByRole('option', { name: /Slack/ }));

  expect(rendered.currentRules().blockedApps).toEqual([SLACK]);
  await waitFor(() => {
    expect(screen.queryByRole('dialog')).toBeNull();
  });
  expect(screen.getByRole('button', { name: 'Remove Slack' })).toBeTruthy();
});

it('closes the picker with Cancel and with Escape without adding anything', async () => {
  const user = userEvent.setup();
  const rendered = renderApps(EMPTY_RULES);
  await user.click(screen.getByRole('button', { name: 'Add app' }));
  await screen.findAllByRole('option');

  await user.click(screen.getByRole('button', { name: 'Cancel' }));
  await waitFor(() => {
    expect(screen.queryByRole('dialog')).toBeNull();
  });
  await user.click(screen.getByRole('button', { name: 'Add app' }));
  await screen.findAllByRole('option');
  await user.keyboard('{Escape}');

  await waitFor(() => {
    expect(screen.queryByRole('dialog')).toBeNull();
  });
  expect(rendered.currentRules().blockedApps).toEqual([]);
});

it('shows a loading state, then the failure with a way to retry', async () => {
  let healthy = false;
  installRulesCommands(invokeMock, {
    list_known_apps: () => {
      if (!healthy) throw new CommandFailure('platform', 'Could not read /Applications');
      return [];
    },
  });
  const user = userEvent.setup();
  renderApps(EMPTY_RULES);

  await user.click(screen.getByRole('button', { name: 'Add app' }));

  expect((await screen.findByRole('alert')).textContent).toBe(
    "Grumpo couldn't list the apps on this Mac.",
  );
  expect(screen.getByText("macOS didn't allow that.")).toBeTruthy();
  expect(screen.queryByText('Could not read /Applications')).toBeNull();
  healthy = true;
  await user.click(screen.getByRole('button', { name: 'Try again' }));
  expect(await screen.findByText('Every app Grumpo found is already on the list.')).toBeTruthy();
});

it('stops adding at the cap and says why', () => {
  const apps = Array.from({ length: RULE_CAPS.apps }, (_, index) => ({
    bundleId: `app.${index.toString()}`,
    name: `App ${index.toString()}`,
  }));
  renderApps({ ...EMPTY_RULES, blockedApps: apps });

  expect(screen.getByText(/You've reached the limit of 50 apps/)).toBeTruthy();
  expect(screen.getByRole('button', { name: 'Add app' }).hasAttribute('disabled')).toBe(true);
});

it('hides add and remove when read only', () => {
  renderApps(SAMPLE_RULES, true);

  expect(screen.queryByRole('button', { name: 'Add app' })).toBeNull();
  expect(screen.queryByRole('button', { name: 'Remove Slack' })).toBeNull();
  expect(screen.getByText('Slack')).toBeTruthy();
});

it('has no accessibility violations, with the picker open too', async () => {
  const user = userEvent.setup();
  renderApps();
  await expectAccessible(document.body);

  await user.click(screen.getByRole('button', { name: 'Add app' }));
  await screen.findAllByRole('option');

  await expectAccessible(document.body);
});
