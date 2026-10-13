import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn<(command: string, ...args: unknown[]) => Promise<unknown>>(),
}));

vi.mock('@/ipc/commands', () => ({ invokeCommand: invokeMock }));

import { LimitsEditor } from '@/features/rules/LimitsEditor';
import { RULE_CAPS } from '@/features/rules/rulesModel';
import type { Rules } from '@/ipc/generated/Rules';
import { EMPTY_RULES, limitUsage, MAIL, SAMPLE_RULES, SLACK } from '@/test/ipc';
import { expectAccessible } from '@/test/render';
import { installRulesCommands, renderEditor } from '@/test/rules';

beforeEach(() => {
  invokeMock.mockReset();
  installRulesCommands(invokeMock);
});

function renderLimits(initial: Rules = SAMPLE_RULES, readOnly = false) {
  return renderEditor((props) => <LimitsEditor {...props} />, { initial, readOnly });
}

it('shows each limit with its value and what was used today', async () => {
  renderLimits();

  expect(await screen.findByText('Used 38min today')).toBeTruthy();
  const spin = screen.getByRole('spinbutton', { name: 'Daily limit for youtube.com' });
  expect(spin.textContent).toBe('30min');
  expect(spin.getAttribute('aria-valuetext')).toBe('30 minutes');
});

it('says a limit was not used yet when there is no usage', async () => {
  installRulesCommands(invokeMock, {
    get_limit_usage: () => [limitUsage({ usedSeconds: 0, limitSeconds: 1_800 })],
  });
  renderLimits();

  expect(await screen.findByText('Not used today')).toBeTruthy();
});

it('shows no usage line while the usage is unknown', () => {
  installRulesCommands(invokeMock, {
    get_limit_usage: () => new Promise(() => undefined),
  });
  renderLimits();

  expect(screen.queryByText(/today/)).toBeNull();
});

it('changes a limit in five minute steps', async () => {
  const user = userEvent.setup();
  const rendered = renderLimits();

  await user.click(
    screen.getByRole('button', { name: 'Increase the daily limit for youtube.com' }),
  );
  await user.click(
    screen.getByRole('button', { name: 'Increase the daily limit for youtube.com' }),
  );
  await user.click(
    screen.getByRole('button', { name: 'Decrease the daily limit for youtube.com' }),
  );

  expect(rendered.currentRules().limits[0]?.dailyMinutes).toBe(35);
  expect(screen.getByRole('spinbutton').textContent).toBe('35min');
});

it('stays inside the range the service accepts, 1 minute to 24 hours', async () => {
  const user = userEvent.setup();
  renderLimits();
  await user.tab();
  await user.tab();
  const spin = screen.getByRole('spinbutton');
  expect(document.activeElement).toBe(spin);

  await user.keyboard('{End}');
  expect(spin.textContent).toBe('24h');
  expect(screen.getByRole('button', { name: /Increase/ }).hasAttribute('disabled')).toBe(true);
  await user.keyboard('{Home}');
  expect(spin.textContent).toBe('1min');
  expect(screen.getByRole('button', { name: /Decrease/ }).hasAttribute('disabled')).toBe(true);
});

it('removes a limit', async () => {
  const user = userEvent.setup();
  const rendered = renderLimits();

  await user.click(screen.getByRole('button', { name: 'Remove the limit for youtube.com' }));

  expect(rendered.currentRules().limits).toEqual([]);
});

it('adds a limit for a website, tidying the address', async () => {
  const user = userEvent.setup();
  const rendered = renderLimits(EMPTY_RULES);
  expect(screen.getByRole('heading', { name: 'No daily limits' })).toBeTruthy();

  await user.click(screen.getByRole('button', { name: 'Add limit' }));
  const dialog = await screen.findByRole('dialog', { name: 'Add a daily limit' });
  await user.type(within(dialog).getByLabelText('Site'), 'https://www.Twitter.com/home');
  await user.click(within(dialog).getByRole('button', { name: 'Increase the daily limit' }));
  await user.click(within(dialog).getByRole('button', { name: 'Add limit' }));

  expect(rendered.currentRules().limits).toEqual([
    { target: { kind: 'site', domain: 'twitter.com' }, dailyMinutes: 35 },
  ]);
  await waitFor(() => {
    expect(screen.queryByRole('dialog')).toBeNull();
  });
  expect(screen.getByRole('spinbutton', { name: 'Daily limit for twitter.com' })).toBeTruthy();
});

it('adds a limit for an app picked from the list', async () => {
  const user = userEvent.setup();
  const rendered = renderLimits(EMPTY_RULES);
  await user.click(screen.getByRole('button', { name: 'Add limit' }));
  const dialog = await screen.findByRole('dialog');

  await user.click(within(dialog).getByRole('radio', { name: 'App' }));
  await user.click(await within(dialog).findByRole('option', { name: /Mail/ }));
  expect(within(dialog).getByText('Mail')).toBeTruthy();
  await user.click(within(dialog).getByRole('button', { name: 'Add limit' }));

  expect(rendered.currentRules().limits).toEqual([
    { target: { kind: 'app', app: MAIL }, dailyMinutes: 30 },
  ]);
  expect(await screen.findByRole('spinbutton', { name: 'Daily limit for Mail' })).toBeTruthy();
});

it('lets the chosen app be changed', async () => {
  const user = userEvent.setup();
  renderLimits(EMPTY_RULES);
  await user.click(screen.getByRole('button', { name: 'Add limit' }));
  const dialog = await screen.findByRole('dialog');
  await user.click(within(dialog).getByRole('radio', { name: 'App' }));
  await user.click(await within(dialog).findByRole('option', { name: /Mail/ }));

  await user.click(within(dialog).getByRole('button', { name: 'Change' }));

  expect(await within(dialog).findByRole('combobox', { name: 'Search apps' })).toBeTruthy();
});

it('asks for an app before adding an app limit', async () => {
  const user = userEvent.setup();
  const rendered = renderLimits(EMPTY_RULES);
  await user.click(screen.getByRole('button', { name: 'Add limit' }));
  const dialog = await screen.findByRole('dialog');

  await user.click(within(dialog).getByRole('radio', { name: 'App' }));
  await user.click(within(dialog).getByRole('button', { name: 'Add limit' }));

  expect(within(dialog).getByRole('alert').textContent).toBe('Choose an app from the list.');
  expect(rendered.currentRules().limits).toEqual([]);
});

it('explains a bad address and a duplicate limit inside the sheet', async () => {
  const user = userEvent.setup();
  renderLimits();
  await user.click(screen.getByRole('button', { name: 'Add limit' }));
  const dialog = await screen.findByRole('dialog');
  const field = within(dialog).getByLabelText('Site');

  await user.type(field, 'nodot{Enter}');
  expect(within(dialog).getByRole('alert').textContent).toBe(
    'Enter a full address with a dot, like youtube.com.',
  );

  await user.clear(field);
  await user.type(field, 'youtube.com{Enter}');
  expect(within(dialog).getByRole('alert').textContent).toBe(
    "There's already a limit for youtube.com.",
  );
});

it('does not offer an app that already has a limit', async () => {
  const user = userEvent.setup();
  renderLimits({
    ...EMPTY_RULES,
    limits: [{ target: { kind: 'app', app: SLACK }, dailyMinutes: 20 }],
  });
  await user.click(screen.getByRole('button', { name: 'Add limit' }));
  const dialog = await screen.findByRole('dialog');
  await user.click(within(dialog).getByRole('radio', { name: 'App' }));

  await within(dialog).findByRole('option', { name: /Safari/ });

  expect(within(dialog).queryByRole('option', { name: /Slack/ })).toBeNull();
  expect(within(dialog).getByRole('option', { name: /Mail/ })).toBeTruthy();
});

it('keeps offering apps whose limit was just removed from the draft', async () => {
  const user = userEvent.setup();
  renderLimits({
    ...EMPTY_RULES,
    limits: [{ target: { kind: 'app', app: SLACK }, dailyMinutes: 20 }],
  });
  await user.click(screen.getByRole('button', { name: 'Remove the limit for Slack' }));
  await user.click(screen.getByRole('button', { name: 'Add limit' }));
  const dialog = await screen.findByRole('dialog');
  await user.click(within(dialog).getByRole('radio', { name: 'App' }));

  expect(await within(dialog).findByRole('option', { name: /Slack/ })).toBeTruthy();
});

it('turns off automatic correction on the website field', async () => {
  const user = userEvent.setup();
  renderLimits(EMPTY_RULES);
  await user.click(screen.getByRole('button', { name: 'Add limit' }));
  const field = within(await screen.findByRole('dialog')).getByLabelText('Site');

  expect(field.getAttribute('autocorrect')).toBe('off');
  expect(field.getAttribute('autocapitalize')).toBe('off');
  expect(field.getAttribute('autocomplete')).toBe('off');
  expect(field.getAttribute('spellcheck')).toBe('false');
});

it('closes the sheet with Cancel', async () => {
  const user = userEvent.setup();
  renderLimits(EMPTY_RULES);
  await user.click(screen.getByRole('button', { name: 'Add limit' }));

  await user.click(await screen.findByRole('button', { name: 'Cancel' }));

  await waitFor(() => {
    expect(screen.queryByRole('dialog')).toBeNull();
  });
});

it('stops adding at the cap and says why', () => {
  const limits = Array.from({ length: RULE_CAPS.limits }, (_, index) => ({
    target: { kind: 'site' as const, domain: `site${index.toString()}.com` },
    dailyMinutes: 30,
  }));
  renderLimits({ ...EMPTY_RULES, limits });

  expect(screen.getByText(/You've reached the limit of 30 daily limits/)).toBeTruthy();
  expect(screen.getByRole('button', { name: 'Add limit' }).hasAttribute('disabled')).toBe(true);
});

it('shows limits without controls when read only', () => {
  renderLimits(SAMPLE_RULES, true);

  expect(screen.queryByRole('button', { name: 'Add limit' })).toBeNull();
  expect(screen.queryByRole('button', { name: /Remove the limit/ })).toBeNull();
  expect(screen.getByRole('spinbutton').getAttribute('aria-disabled')).toBe('true');
});

it('has no accessibility violations, with the sheet open too', async () => {
  const user = userEvent.setup();
  renderLimits();
  await screen.findByText('Used 38min today');
  await expectAccessible(document.body);

  await user.click(screen.getByRole('button', { name: 'Add limit' }));
  await screen.findByRole('dialog');

  await expectAccessible(document.body);
});
