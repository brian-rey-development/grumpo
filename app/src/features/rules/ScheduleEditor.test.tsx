import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn<(command: string, ...args: unknown[]) => Promise<unknown>>(),
}));

vi.mock('@/ipc/commands', () => ({ invokeCommand: invokeMock }));

import { RULE_CAPS } from '@/features/rules/rulesModel';
import { ScheduleEditor } from '@/features/rules/ScheduleEditor';
import type { Rules } from '@/ipc/generated/Rules';
import { EMPTY_RULES, SAMPLE_RULES } from '@/test/ipc';
import { expectAccessible } from '@/test/render';
import { installRulesCommands, renderEditor } from '@/test/rules';

beforeEach(() => {
  invokeMock.mockReset();
  installRulesCommands(invokeMock);
});

function renderSchedule(initial: Rules = SAMPLE_RULES, readOnly = false) {
  return renderEditor((props) => <ScheduleEditor {...props} />, { initial, readOnly });
}

function timeOf(label: string): string {
  const select = screen.getByRole<HTMLSelectElement>('combobox', { name: label });
  return select.selectedOptions[0]?.textContent ?? '';
}

it('shows each focus time with its days and times', () => {
  renderSchedule();

  const row = screen.getByRole('group', { name: 'Focus time 1' });
  const pressed = within(row)
    .getAllByRole('button', { pressed: true })
    .map((button) => button.getAttribute('aria-label'));
  expect(pressed).toEqual(['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday']);
  expect(timeOf('Start time for focus time 1')).toBe('09:00');
  expect(timeOf('End time for focus time 1')).toBe('17:00');
});

it('explains an empty schedule and adds Monday to Friday, 09:00 to 17:00', async () => {
  const user = userEvent.setup();
  const rendered = renderSchedule(EMPTY_RULES);
  expect(screen.getByRole('heading', { name: 'No focus times' })).toBeTruthy();

  await user.click(screen.getByRole('button', { name: 'Add focus time' }));

  expect(rendered.currentRules().schedule.windows).toEqual([
    { days: ['mon', 'tue', 'wed', 'thu', 'fri'], start: 540, end: 1020 },
  ]);
  expect(screen.getByRole('group', { name: 'Focus time 1' })).toBeTruthy();
});

it('shows the example time in the clock of the locale', () => {
  renderSchedule(EMPTY_RULES);

  expect(
    screen.getByText(/Add a time like Monday to Friday, 09:00 to 17:00, and Grumpo/),
  ).toBeTruthy();
});

it('changes days and times', async () => {
  const user = userEvent.setup();
  const rendered = renderSchedule();
  const row = screen.getByRole('group', { name: 'Focus time 1' });

  await user.click(within(row).getByRole('button', { name: 'Saturday' }));
  await user.click(within(row).getByRole('button', { name: 'Monday' }));
  await user.selectOptions(
    screen.getByRole('combobox', { name: 'Start time for focus time 1' }),
    '08:30',
  );
  await user.selectOptions(
    screen.getByRole('combobox', { name: 'End time for focus time 1' }),
    'End of day',
  );

  expect(rendered.currentRules().schedule.windows).toEqual([
    { days: ['tue', 'wed', 'thu', 'fri', 'sat'], start: 510, end: 1440 },
  ]);
  expect(timeOf('End time for focus time 1')).toBe('End of day');
});

it('removes a focus time', async () => {
  const user = userEvent.setup();
  const rendered = renderSchedule();

  await user.click(screen.getByRole('button', { name: 'Remove focus time 1' }));

  expect(rendered.currentRules().schedule.windows).toEqual([]);
});

it('marks a focus time without days', async () => {
  const user = userEvent.setup();
  renderSchedule({
    ...EMPTY_RULES,
    schedule: { windows: [{ days: ['mon'], start: 540, end: 1020 }] },
  });

  await user.click(screen.getByRole('button', { name: 'Monday' }));

  expect(screen.getByRole('alert').textContent).toBe('Choose at least one day.');
});

it('links a missing-days message to the days it is about', async () => {
  const user = userEvent.setup();
  renderSchedule({
    ...EMPTY_RULES,
    schedule: { windows: [{ days: ['mon'], start: 540, end: 1020 }] },
  });

  await user.click(screen.getByRole('button', { name: 'Monday' }));

  const days = screen.getByRole('group', { name: 'Days for focus time 1' });
  const message = screen.getByRole('alert');
  expect(days.getAttribute('aria-describedby')).toBe(message.id);
  expect(message.id).not.toBe('');
  const start = screen.getByRole('combobox', { name: 'Start time for focus time 1' });
  expect(start.getAttribute('aria-describedby')).toBeNull();
  expect(start.getAttribute('aria-invalid')).toBeNull();
});

it('marks an end that comes before the start', async () => {
  const user = userEvent.setup();
  renderSchedule();

  await user.selectOptions(
    screen.getByRole('combobox', { name: 'End time for focus time 1' }),
    '08:00',
  );

  expect(screen.getByRole('alert').textContent).toBe('The end has to come after the start.');
  expect(
    screen
      .getByRole('combobox', { name: 'Start time for focus time 1' })
      .getAttribute('aria-invalid'),
  ).toBe('true');
});

it('links an end-before-start message to both times and leaves the days alone', async () => {
  const user = userEvent.setup();
  renderSchedule();

  await user.selectOptions(
    screen.getByRole('combobox', { name: 'End time for focus time 1' }),
    '08:00',
  );

  const message = screen.getByRole('alert');
  for (const name of ['Start time for focus time 1', 'End time for focus time 1']) {
    const time = screen.getByRole('combobox', { name });
    expect(time.getAttribute('aria-invalid')).toBe('true');
    expect(time.getAttribute('aria-describedby')).toBe(message.id);
  }
  const days = screen.getByRole('group', { name: 'Days for focus time 1' });
  expect(days.getAttribute('aria-describedby')).toBeNull();
});

it('draws the week and describes it in words', () => {
  renderSchedule();

  const preview = screen.getByRole('img');
  const label = preview.getAttribute('aria-label') ?? '';
  expect(label).toContain('Monday 09:00 to 17:00.');
  expect(label).toContain('Friday 09:00 to 17:00.');
  expect(label).toContain('Saturday no focus time.');
  expect(preview.querySelectorAll('[style*="left"]').length).toBeGreaterThanOrEqual(5);
});

it('stops adding at the cap and says why', () => {
  const windows = Array.from({ length: RULE_CAPS.windows }, () => ({
    days: ['mon' as const],
    start: 60,
    end: 120,
  }));
  renderSchedule({ ...EMPTY_RULES, schedule: { windows } });

  expect(screen.getByText(/You've reached the limit of 50 focus times/)).toBeTruthy();
  expect(screen.getByRole('button', { name: 'Add focus time' }).hasAttribute('disabled')).toBe(
    true,
  );
});

it('disables every control when read only', () => {
  renderSchedule(SAMPLE_RULES, true);

  expect(screen.queryByRole('button', { name: 'Add focus time' })).toBeNull();
  expect(screen.queryByRole('button', { name: 'Remove focus time 1' })).toBeNull();
  expect(screen.getByRole('button', { name: 'Monday' }).hasAttribute('disabled')).toBe(true);
  expect(
    screen.getByRole('combobox', { name: 'Start time for focus time 1' }).hasAttribute('disabled'),
  ).toBe(true);
});

it('has no accessibility violations', async () => {
  const rendered = renderSchedule();

  await expectAccessible(rendered.container);
});
