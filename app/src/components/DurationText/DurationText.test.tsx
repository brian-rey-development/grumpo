import { render, screen } from '@testing-library/react';
import { expect, it } from 'vitest';

import { DurationText } from '@/components/DurationText/DurationText';

it.each([
  [15_120, '4h 12min'],
  [2_280, '38min'],
  [30, '< 1min'],
  [0, '0min'],
])('exposes %i seconds to assistive technology as %s', (seconds, text) => {
  render(<DurationText seconds={seconds} />);

  expect(screen.getByText(text)).toBeTruthy();
});

it.each([
  [15_120, '4h12min'],
  [3_600, '1h'],
  [30, '< 1min'],
])('draws %i seconds with the unit set apart from the digits as %s', (seconds, text) => {
  const { container } = render(<DurationText seconds={seconds} />);

  expect(container.querySelector('[aria-hidden="true"]')?.textContent).toBe(text);
});

it('shows the new value at once when it changes', () => {
  const { container, rerender } = render(<DurationText seconds={1_800} />);

  rerender(<DurationText seconds={600} />);

  expect(container.querySelector('[aria-hidden="true"]')?.textContent).toBe('10min');
});
