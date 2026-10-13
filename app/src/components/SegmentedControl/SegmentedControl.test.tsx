import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { expect, it } from 'vitest';

import { SegmentedControl } from '@/components/SegmentedControl/SegmentedControl';

const OPTIONS = [
  { value: '30', label: '30 min' },
  { value: '60', label: '60 min' },
  { value: '90', label: '90 min' },
] as const;

function Controlled() {
  const [value, setValue] = useState<'30' | '60' | '90'>('30');
  return <SegmentedControl label="Duration" options={OPTIONS} value={value} onChange={setValue} />;
}

it('marks the selected option and keeps a single tab stop', () => {
  render(<Controlled />);

  const radios = screen.getAllByRole('radio');
  expect(radios.map((radio) => radio.getAttribute('aria-checked'))).toEqual([
    'true',
    'false',
    'false',
  ]);
  expect(radios.map((radio) => radio.tabIndex)).toEqual([0, -1, -1]);
  expect(screen.getByRole('radiogroup', { name: 'Duration' })).toBeTruthy();
});

it('moves selection and focus with the arrow keys, wrapping at the ends', async () => {
  const user = userEvent.setup();
  render(<Controlled />);
  await user.tab();

  await user.keyboard('{ArrowRight}');
  expect(screen.getByRole('radio', { name: '60 min' }).getAttribute('aria-checked')).toBe('true');
  expect(document.activeElement).toBe(screen.getByRole('radio', { name: '60 min' }));

  await user.keyboard('{ArrowLeft}{ArrowLeft}');
  expect(screen.getByRole('radio', { name: '90 min' }).getAttribute('aria-checked')).toBe('true');

  await user.keyboard('{Home}');
  expect(screen.getByRole('radio', { name: '30 min' }).getAttribute('aria-checked')).toBe('true');
  await user.keyboard('{End}');
  expect(screen.getByRole('radio', { name: '90 min' }).getAttribute('aria-checked')).toBe('true');
});

it('selects an option on click', async () => {
  const user = userEvent.setup();
  render(<Controlled />);

  await user.click(screen.getByRole('radio', { name: '90 min' }));

  expect(screen.getByRole('radio', { name: '90 min' }).getAttribute('aria-checked')).toBe('true');
});
