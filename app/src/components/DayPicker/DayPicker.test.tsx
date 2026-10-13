import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect, it, vi } from 'vitest';

import { DayPicker } from '@/components/DayPicker/DayPicker';
import { dayName, sortDays, WEEK_DAYS } from '@/lib/weekDays';
import { expectAccessible } from '@/test/render';

it('names every day in full and shows one letter', () => {
  render(<DayPicker label="Days" value={['mon']} locale="en-GB" onChange={() => undefined} />);

  const group = screen.getByRole('group', { name: 'Days' });
  const buttons = screen.getAllByRole('button');
  expect(buttons.map((button) => button.getAttribute('aria-label'))).toEqual([
    'Monday',
    'Tuesday',
    'Wednesday',
    'Thursday',
    'Friday',
    'Saturday',
    'Sunday',
  ]);
  expect(buttons.map((button) => button.textContent)).toEqual(['M', 'T', 'W', 'T', 'F', 'S', 'S']);
  expect(group.contains(buttons[0] ?? null)).toBe(true);
});

it('localizes the labels', () => {
  render(<DayPicker label="Dias" value={[]} locale="es" onChange={() => undefined} />);

  expect(screen.getByRole('button', { name: 'miércoles' }).textContent).toBe('X');
});

it('marks selected days as pressed', () => {
  render(
    <DayPicker label="Days" value={['tue', 'sat']} locale="en-GB" onChange={() => undefined} />,
  );

  const pressed = screen
    .getAllByRole('button', { pressed: true })
    .map((button) => button.getAttribute('aria-label'));
  expect(pressed).toEqual(['Tuesday', 'Saturday']);
});

it('adds and removes days keeping the week order', async () => {
  const onChange = vi.fn();
  const user = userEvent.setup();
  render(<DayPicker label="Days" value={['fri', 'mon']} locale="en-GB" onChange={onChange} />);

  await user.click(screen.getByRole('button', { name: 'Wednesday' }));
  await user.click(screen.getByRole('button', { name: 'Monday' }));

  expect(onChange.mock.calls).toEqual([[['mon', 'wed', 'fri']], [['fri']]]);
});

it('does not react when disabled', async () => {
  const onChange = vi.fn();
  const user = userEvent.setup();
  render(<DayPicker label="Days" value={[]} locale="en-GB" disabled onChange={onChange} />);

  await user.click(screen.getByRole('button', { name: 'Monday' }));

  expect(onChange).not.toHaveBeenCalled();
});

it('builds day names from a fixed Monday first week', () => {
  expect(WEEK_DAYS).toHaveLength(7);
  expect(dayName('sun', 'long', 'en-GB')).toBe('Sunday');
  expect(dayName('mon', 'short', 'en-GB')).toBe('Mon');
  expect(sortDays(['sun', 'mon'])).toEqual(['mon', 'sun']);
});

it('has no accessibility violations', async () => {
  const { container } = render(
    <DayPicker label="Days" value={['mon']} locale="en-GB" onChange={() => undefined} />,
  );

  await expectAccessible(container);
});
