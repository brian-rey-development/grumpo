import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { expect, it, vi } from 'vitest';

import { Stepper } from '@/components/Stepper/Stepper';
import { stepDown, stepUp, valueForKey } from '@/components/Stepper/stepping';
import { expectAccessible } from '@/test/render';

const RANGE = { min: 5, max: 600, step: 5 };

function Controlled({
  initial = 30,
  onChange,
}: {
  initial?: number;
  onChange?: (v: number) => void;
}) {
  const [value, setValue] = useState(initial);
  return (
    <Stepper
      label="Daily limit"
      decreaseLabel="Decrease daily limit"
      increaseLabel="Increase daily limit"
      value={value}
      {...RANGE}
      formatValue={(minutes) => `${minutes.toString()}m`}
      formatSpoken={(minutes) => `${minutes.toString()} minutes`}
      onChange={(next) => {
        onChange?.(next);
        setValue(next);
      }}
    />
  );
}

it('exposes a spin button with range and spoken value', () => {
  render(<Controlled />);

  const spin = screen.getByRole('spinbutton', { name: 'Daily limit' });
  expect(spin.getAttribute('aria-valuenow')).toBe('30');
  expect(spin.getAttribute('aria-valuemin')).toBe('5');
  expect(spin.getAttribute('aria-valuemax')).toBe('600');
  expect(spin.getAttribute('aria-valuetext')).toBe('30 minutes');
  expect(spin.textContent).toBe('30m');
});

it('steps with the buttons', async () => {
  const user = userEvent.setup();
  render(<Controlled />);

  await user.click(screen.getByRole('button', { name: 'Increase daily limit' }));
  await user.click(screen.getByRole('button', { name: 'Increase daily limit' }));
  await user.click(screen.getByRole('button', { name: 'Decrease daily limit' }));

  expect(screen.getByRole('spinbutton').textContent).toBe('35m');
});

it('steps with the arrow, page, home and end keys', async () => {
  const user = userEvent.setup();
  render(<Controlled />);
  await user.tab();
  expect(document.activeElement).toBe(screen.getByRole('spinbutton'));

  await user.keyboard('{ArrowUp}');
  expect(screen.getByRole('spinbutton').textContent).toBe('35m');
  await user.keyboard('{ArrowDown}{ArrowDown}');
  expect(screen.getByRole('spinbutton').textContent).toBe('25m');
  await user.keyboard('{PageUp}');
  expect(screen.getByRole('spinbutton').textContent).toBe('55m');
  await user.keyboard('{End}');
  expect(screen.getByRole('spinbutton').textContent).toBe('600m');
  await user.keyboard('{Home}');
  expect(screen.getByRole('spinbutton').textContent).toBe('5m');
});

it('stops at the bounds and disables the matching button', () => {
  const { rerender } = render(<Controlled initial={5} />);
  expect(
    screen.getByRole('button', { name: 'Decrease daily limit' }).hasAttribute('disabled'),
  ).toBe(true);

  rerender(<Controlled initial={600} key="max" />);
  expect(
    screen.getByRole('button', { name: 'Increase daily limit' }).hasAttribute('disabled'),
  ).toBe(true);
});

it('ignores keys when disabled', async () => {
  const onChange = vi.fn();
  const user = userEvent.setup();
  render(
    <Stepper
      label="Limit"
      decreaseLabel="Decrease limit"
      increaseLabel="Increase limit"
      value={30}
      {...RANGE}
      disabled
      formatValue={String}
      onChange={onChange}
    />,
  );

  await user.keyboard('{ArrowUp}');
  await user.click(screen.getByRole('button', { name: 'Increase limit', hidden: true }));

  expect(onChange).not.toHaveBeenCalled();
});

it('snaps off-grid and out-of-range values onto the grid', () => {
  expect(stepUp(7, RANGE)).toBe(10);
  expect(stepDown(7, RANGE)).toBe(5);
  expect(stepUp(1, RANGE)).toBe(5);
  expect(stepDown(700, RANGE)).toBe(600);
  expect(stepUp(600, RANGE)).toBe(600);
  expect(valueForKey('x', 30, RANGE)).toBeNull();
  expect(valueForKey('PageDown', 30, RANGE)).toBe(5);
});

it('has no accessibility violations', async () => {
  const { container } = render(<Controlled />);

  await expectAccessible(container);
});

it('steps on multiples of the step, so a minimum off the grid and the maximum stay reachable', () => {
  const wide = { min: 1, max: 1440, step: 5 };

  expect(stepUp(1, wide)).toBe(5);
  expect(stepDown(5, wide)).toBe(1);
  expect(stepDown(1, wide)).toBe(1);
  expect(stepUp(1435, wide)).toBe(1440);
  expect(stepUp(1440, wide)).toBe(1440);
  expect(stepUp(30, wide)).toBe(35);
  expect(valueForKey('End', 30, wide)).toBe(1440);
  expect(valueForKey('Home', 30, wide)).toBe(1);
});
