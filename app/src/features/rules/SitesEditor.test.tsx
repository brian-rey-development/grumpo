import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn<(command: string, ...args: unknown[]) => Promise<unknown>>(),
}));

vi.mock('@/ipc/commands', () => ({ invokeCommand: invokeMock }));

import { SitesEditor } from '@/features/rules/SitesEditor';
import { RULE_CAPS } from '@/features/rules/rulesModel';
import type { Rules } from '@/ipc/generated/Rules';
import { EMPTY_RULES, SAMPLE_RULES } from '@/test/ipc';
import { expectAccessible } from '@/test/render';
import { installRulesCommands, renderEditor } from '@/test/rules';

beforeEach(() => {
  invokeMock.mockReset();
  installRulesCommands(invokeMock);
});

function renderSites(initial: Rules = SAMPLE_RULES, readOnly = false) {
  return renderEditor((props) => <SitesEditor {...props} />, { initial, readOnly });
}

it('lists the blocked sites with a remove button each', () => {
  renderSites();

  expect(screen.getByRole('list', { name: 'Blocked sites' })).toBeTruthy();
  expect(screen.getAllByRole('listitem').map((item) => item.textContent)).toEqual([
    'reddit.com',
    'youtube.com',
  ]);
  expect(screen.getByRole('button', { name: 'Remove reddit.com' })).toBeTruthy();
});

it('promises the site together with its www and mobile versions, and nothing more', () => {
  renderSites();

  expect(
    screen.getByText('Blocked during focus time, together with their www and mobile versions.'),
  ).toBeTruthy();
  expect(screen.queryByText(/subdomain/i)).toBeNull();
  expect(screen.queryByText(/tab/i)).toBeNull();
});

async function openSheet(user: ReturnType<typeof userEvent.setup>) {
  await user.click(screen.getByRole('button', { name: 'Add site' }));
  return within(screen.getByRole('dialog', { name: 'Add a site' }));
}

it.each([
  [
    'a public suffix',
    'co.uk',
    'That ending is shared by many sites, like co.uk. Enter a specific site under it.',
  ],
  ['an IP address', '192.168.0.1', "IP addresses can't be blocked here. Enter a site name."],
])('refuses %s the way the service does', async (_, input, message) => {
  const user = userEvent.setup();
  renderSites(EMPTY_RULES);
  const sheet = await openSheet(user);

  await user.type(sheet.getByLabelText('Site'), `${input}{Enter}`);

  expect(sheet.getByRole('alert').textContent).toBe(message);
});

it('explains an empty list', () => {
  renderSites(EMPTY_RULES);

  expect(screen.getByRole('heading', { name: 'No blocked sites' })).toBeTruthy();
  expect(screen.getByText(/Add a site like youtube.com/)).toBeTruthy();
});

it('adds a site after tidying what was typed and closes the sheet', async () => {
  const user = userEvent.setup();
  const rendered = renderSites(EMPTY_RULES);
  const sheet = await openSheet(user);

  await user.type(sheet.getByLabelText('Site'), 'https://www.Example.COM/some/path{Enter}');

  expect(rendered.currentRules().blockedSites).toEqual(['example.com']);
  await waitFor(() => {
    expect(screen.queryByRole('dialog')).toBeNull();
  });
  expect(screen.getByText('example.com')).toBeTruthy();
});

it('keeps the sites sorted as they are added', async () => {
  const user = userEvent.setup();
  const rendered = renderSites();
  const sheet = await openSheet(user);

  await user.type(sheet.getByLabelText('Site'), 'news.com');
  await user.click(sheet.getByRole('button', { name: 'Add site' }));

  expect(rendered.currentRules().blockedSites).toEqual(['news.com', 'reddit.com', 'youtube.com']);
});

it('asks for a dot and keeps the text so it can be fixed', async () => {
  const user = userEvent.setup();
  const rendered = renderSites(EMPTY_RULES);
  const sheet = await openSheet(user);

  await user.type(sheet.getByLabelText('Site'), 'youtube{Enter}');

  const alert = sheet.getByRole('alert');
  expect(alert.textContent).toBe('Enter a full address with a dot, like youtube.com.');
  expect(sheet.getByLabelText('Site').getAttribute('aria-describedby')).toBe(alert.id);
  expect(sheet.getByLabelText<HTMLInputElement>('Site').value).toBe('youtube');
  expect(rendered.currentRules().blockedSites).toEqual([]);

  await user.type(sheet.getByLabelText('Site'), '.com');
  expect(sheet.queryByRole('alert')).toBeNull();
});

it('refuses a site that is already blocked', async () => {
  const user = userEvent.setup();
  renderSites();
  const sheet = await openSheet(user);

  await user.type(sheet.getByLabelText('Site'), 'www.reddit.com{Enter}');

  expect(sheet.getByRole('alert').textContent).toBe('reddit.com is already blocked.');
});

it('puts the cursor in the field when the sheet opens', async () => {
  const user = userEvent.setup();
  renderSites(EMPTY_RULES);
  const sheet = await openSheet(user);

  await waitFor(() => {
    expect(document.activeElement).toBe(sheet.getByLabelText('Site'));
  });
});

it('removes a site', async () => {
  const user = userEvent.setup();
  const rendered = renderSites();

  await user.click(screen.getByRole('button', { name: 'Remove reddit.com' }));

  expect(rendered.currentRules().blockedSites).toEqual(['youtube.com']);
  expect(screen.queryByText('reddit.com')).toBeNull();
});

it('stops adding at the cap and says why', () => {
  const full: Rules = {
    ...EMPTY_RULES,
    blockedSites: Array.from(
      { length: RULE_CAPS.sites },
      (_, index) => `site${index.toString()}.com`,
    ),
  };
  renderSites(full);

  expect(screen.getByText(/You've reached the limit of 500 sites/)).toBeTruthy();
  expect(screen.getByRole('button', { name: 'Add site' }).hasAttribute('disabled')).toBe(true);
});

it('shows the list without any way to change it when read only', () => {
  renderSites(SAMPLE_RULES, true);

  expect(screen.queryByRole('button', { name: 'Add site' })).toBeNull();
  expect(screen.queryByRole('button', { name: /Remove/ })).toBeNull();
  expect(screen.getByText('reddit.com')).toBeTruthy();
});

it('has no accessibility violations', async () => {
  const rendered = renderSites();

  await expectAccessible(rendered.container);
});
