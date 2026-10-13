import { render, renderHook, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeAll, expect, it } from 'vitest';

import { BlockScreenView } from '@/blocker/BlockScreenView';
import { ColumnChart } from '@/components/ColumnChart/ColumnChart';
import { DurationText } from '@/components/DurationText/DurationText';
import { spring, useMotionTransition } from '@/design/motion';
import { AppShell } from '@/layout/AppShell';
import { mockMatchMedia } from '@/test/matchMedia';
import { renderWithProviders } from '@/test/render';

const VALUES = [0, 1_800, 3_600, 0];

beforeAll(() => {
  mockMatchMedia({ reducedMotion: true });
});

it('swaps every spring and tween for an instant transition', () => {
  const { result } = renderHook(() => useMotionTransition(spring.gentle));

  expect(result.current).toEqual({ duration: 0 });
});

it('shows a new duration at once instead of counting towards it', () => {
  const { container, rerender } = render(<DurationText seconds={1_800} />);

  rerender(<DurationText seconds={15_120} />);

  expect(container.querySelector('[aria-hidden="true"]')?.textContent).toBe('4h12min');
});

it('reveals every column at once without any growing or stagger', async () => {
  renderWithProviders(
    <ColumnChart
      label="Screen time by hour"
      datasetKey="reduced"
      values={VALUES}
      labelFor={String}
      formatValue={String}
      ticks={[]}
      emptyText="Nothing tracked yet"
    />,
  );

  await waitFor(
    () => {
      const transforms = screen
        .getAllByRole('option')
        .map((option) => option.firstElementChild?.getAttribute('style') ?? '');
      expect(transforms.every((style) => !style.includes('translateY(100%)'))).toBe(true);
    },
    { timeout: 200 },
  );
});

it('swaps screens without waiting for a crossfade', async () => {
  const user = userEvent.setup();
  const { container } = renderWithProviders(
    <AppShell>
      <p>content</p>
    </AppShell>,
  );

  await user.click(screen.getByRole('button', { name: 'Week' }));

  await waitFor(
    () => {
      expect(container.querySelectorAll('[data-screen]')).toHaveLength(1);
    },
    { timeout: 100 },
  );
});

it('puts the block screen mascot at rest at once instead of squashing and shaking it', async () => {
  const { container } = render(
    <BlockScreenView
      screen={{
        id: 1,
        target: { kind: 'site', domain: 'youtube.com' },
        reason: 'schedule',
        until: null,
        strict: false,
      }}
      onDismiss={() => undefined}
    />,
  );
  const stage = container.querySelector('img')?.parentElement?.parentElement;

  await waitFor(
    () => {
      expect(stage?.getAttribute('style') ?? '').not.toContain('scaleY');
      expect(stage?.style.transform).toBe('none');
    },
    { timeout: 200 },
  );
});
