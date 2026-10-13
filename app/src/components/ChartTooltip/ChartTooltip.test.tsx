import { render, screen } from '@testing-library/react';
import { expect, it } from 'vitest';

import { ChartTooltip } from '@/components/ChartTooltip/ChartTooltip';

it.each([
  [0, '12%'],
  [100, '88%'],
  [50, '50%'],
])(
  'keeps an anchor of %i%% away from both edges so the bubble stays inside its chart',
  (anchor, left) => {
    render(<ChartTooltip anchorPercent={anchor}>Tip</ChartTooltip>);

    expect(screen.getByText('Tip').style.left).toBe(left);
  },
);

it('places the bubble at the clamped anchor and hides it from assistive technology', () => {
  render(<ChartTooltip anchorPercent={2}>09:00 to 10:00, 38m</ChartTooltip>);

  const tooltip = screen.getByText('09:00 to 10:00, 38m');
  expect(tooltip.style.left).toBe('12%');
  expect(tooltip.getAttribute('aria-hidden')).toBe('true');
});
