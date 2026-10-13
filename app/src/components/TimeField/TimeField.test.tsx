import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect, it, vi } from 'vitest';

import { TimeField } from '@/components/TimeField/TimeField';
import { expectAccessible } from '@/test/render';

function optionLabels(): string[] {
  return screen.getAllByRole('option').map((option) => option.textContent);
}

it('lists quarter hours in the locale clock and selects the current value', () => {
  render(<TimeField label="Start time" value={540} onChange={() => undefined} />);

  const select = screen.getByRole('combobox', { name: 'Start time' });
  expect(select instanceof HTMLSelectElement && select.value).toBe('540');
  expect(optionLabels().slice(0, 3)).toEqual(['00:00', '00:15', '00:30']);
  expect(optionLabels().at(-1)).toBe('23:45');
  expect(screen.getByRole('option', { name: '09:00' })).toBeTruthy();
});

it('offers End of day only when asked to', () => {
  const { rerender } = render(
    <TimeField label="End time" value={1020} onChange={() => undefined} />,
  );
  expect(screen.queryByRole('option', { name: 'End of day' })).toBeNull();

  rerender(
    <TimeField
      label="End time"
      value={1440}
      endOfDayLabel="End of day"
      onChange={() => undefined}
    />,
  );
  const select = screen.getByRole('combobox', { name: 'End time' });
  expect(select instanceof HTMLSelectElement && select.value).toBe('1440');
  expect(optionLabels().at(-1)).toBe('End of day');
});

it('keeps a value that falls between steps selectable', () => {
  render(<TimeField label="Start time" value={547} onChange={() => undefined} />);

  expect(screen.getByRole('option', { name: '09:07' })).toBeTruthy();
});

it('starts the options at the minimum', () => {
  render(
    <TimeField
      label="End time"
      value={600}
      min={555}
      endOfDayLabel="End of day"
      onChange={() => undefined}
    />,
  );

  expect(optionLabels()[0]).toBe('09:15');
});

it('reports the chosen minutes', async () => {
  const onChange = vi.fn();
  const user = userEvent.setup();
  render(<TimeField label="Start time" value={540} onChange={onChange} />);

  await user.selectOptions(screen.getByRole('combobox', { name: 'Start time' }), '10:30');

  expect(onChange).toHaveBeenCalledExactlyOnceWith(630);
});

it('flags an invalid value and can be disabled', () => {
  render(<TimeField label="End time" value={540} invalid disabled onChange={() => undefined} />);

  const select = screen.getByRole('combobox', { name: 'End time' });
  expect(select.getAttribute('aria-invalid')).toBe('true');
  expect(select instanceof HTMLSelectElement && select.disabled).toBe(true);
});

it('has no accessibility violations', async () => {
  const { container } = render(
    <>
      <TimeField label="Start time" value={540} onChange={() => undefined} />
      <TimeField
        label="End time"
        value={1440}
        endOfDayLabel="End of day"
        onChange={() => undefined}
      />
    </>,
  );

  await expectAccessible(container);
});
