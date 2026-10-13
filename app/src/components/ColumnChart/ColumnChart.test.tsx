import { act, fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, expect, it, vi } from 'vitest';

import { ColumnChart } from '@/components/ColumnChart/ColumnChart';
import { REVEAL_SETTLE_MS, RevealProvider } from '@/components/Reveal/Reveal';
import { expectAccessible, renderWithProviders } from '@/test/render';

const HOUR_SECONDS = 3_600;
const HOURLY = Array.from({ length: 24 }, (_, hour) =>
  hour === 9 ? 2_280 : hour === 14 ? HOUR_SECONDS : 0,
);
const TICKS = [0, 6, 12, 18].map((index) => ({ index, label: index.toString().padStart(2, '0') }));

afterEach(() => {
  vi.useRealTimers();
});

function pad(hour: number): string {
  return hour.toString().padStart(2, '0');
}

interface HourlyProps {
  hourly?: number[];
  highlightIndex?: number;
}

function Hourly({ hourly = HOURLY, highlightIndex }: HourlyProps) {
  return (
    <ColumnChart
      label="Activity by hour"
      datasetKey="hourly:2026-10-12"
      values={hourly}
      labelFor={(hour) => `${pad(hour)}:00`}
      formatValue={(seconds) => `${(seconds / 60).toString()} min`}
      ticks={TICKS}
      highlightIndex={highlightIndex}
      emptyText="Nothing tracked yet"
    />
  );
}

it('is one labelled listbox with a single tab stop and an option per column', () => {
  renderWithProviders(<Hourly />);

  const listbox = screen.getByRole('listbox', { name: 'Activity by hour' });
  expect(listbox.tabIndex).toBe(0);
  const options = screen.getAllByRole('option');
  expect(options).toHaveLength(24);
  expect(options.filter((option) => option.tabIndex === 0)).toHaveLength(0);
  expect(screen.getByRole('option', { name: '09:00, 38 min' })).toBeTruthy();
  expect(screen.getByRole('option', { name: '14:00, 60 min' })).toBeTruthy();
  expect(screen.getByRole('option', { name: '23:00, 0 min' })).toBeTruthy();
});

it('starts the keyboard on the highlighted column when there is one', async () => {
  const user = userEvent.setup();
  renderWithProviders(<Hourly highlightIndex={14} />);

  await user.tab();

  const listbox = screen.getByRole('listbox');
  expect(listbox.getAttribute('aria-activedescendant')).toBe(
    screen.getByRole('option', { name: '14:00, 60 min' }).id,
  );
});

it('moves through the columns with the arrow keys, Home and End', async () => {
  const user = userEvent.setup();
  renderWithProviders(<Hourly />);
  const listbox = screen.getByRole('listbox');

  await user.tab();
  expect(document.activeElement).toBe(listbox);
  expect(listbox.getAttribute('aria-activedescendant')).toBe(
    screen.getByRole('option', { name: /^00:00/ }).id,
  );

  await user.keyboard('{ArrowRight}{ArrowRight}');
  expect(listbox.getAttribute('aria-activedescendant')).toBe(
    screen.getByRole('option', { name: /^02:00/ }).id,
  );

  await user.keyboard('{End}{ArrowRight}');
  expect(listbox.getAttribute('aria-activedescendant')).toBe(
    screen.getByRole('option', { name: /^23:00/ }).id,
  );

  await user.keyboard('{Home}{ArrowLeft}');
  expect(listbox.getAttribute('aria-activedescendant')).toBe(
    screen.getByRole('option', { name: /^00:00/ }).id,
  );
  expect(screen.getByRole('option', { name: /^00:00/ }).getAttribute('aria-selected')).toBe('true');
});

it('lets go of the column when focus leaves the chart', async () => {
  const user = userEvent.setup();
  renderWithProviders(
    <>
      <Hourly />
      <button type="button">After</button>
    </>,
  );
  await user.tab();
  await user.keyboard('{ArrowRight}{ArrowRight}{ArrowRight}');

  await user.tab();

  expect(document.activeElement).toBe(screen.getByRole('button', { name: 'After' }));
  expect(screen.getByRole('listbox').getAttribute('aria-activedescendant')).toBeNull();
});

it('reveals the exact value on keyboard focus and hover, and hides it again', async () => {
  const user = userEvent.setup();
  renderWithProviders(<Hourly highlightIndex={9} />);
  const tooltip = (name: string) => screen.queryByText(name, { selector: '[aria-hidden="true"]' });

  await user.tab();
  expect(tooltip('09:00, 38 min')).toBeTruthy();
  await user.keyboard('{Escape}');
  expect(tooltip('09:00, 38 min')).toBeNull();

  const column = screen.getByRole('option', { name: '14:00, 60 min' });
  fireEvent.pointerEnter(column);
  expect(tooltip('14:00, 60 min')).toBeTruthy();
  fireEvent.pointerLeave(column);
  expect(tooltip('14:00, 60 min')).toBeNull();
});

it('keeps the tooltip inside the chart for the first and last columns', () => {
  renderWithProviders(<Hourly />);
  const tooltipLeft = (name: RegExp) =>
    screen.getByText(name, { selector: '[aria-hidden="true"]' }).style.left;

  fireEvent.pointerEnter(screen.getByRole('option', { name: /^00:00/ }));
  expect(tooltipLeft(/^00:00/)).toBe('12%');
  fireEvent.pointerLeave(screen.getByRole('option', { name: /^00:00/ }));

  fireEvent.pointerEnter(screen.getByRole('option', { name: /^23:00/ }));
  expect(tooltipLeft(/^23:00/)).toBe('88%');
});

it('tolerates short data and shows the empty message when nothing was tracked', () => {
  renderWithProviders(<Hourly hourly={[]} />);

  expect(screen.getByText('Nothing tracked yet')).toBeTruthy();
  expect(screen.queryAllByRole('option')).toHaveLength(0);
});

it('draws a tick label at each requested column', () => {
  renderWithProviders(<Hourly />);

  for (const label of ['00', '06', '12', '18']) {
    expect(screen.getByText(label)).toBeTruthy();
  }
});

it('marks the highlighted column', () => {
  renderWithProviders(<Hourly highlightIndex={14} />);

  const marked = screen
    .getAllByRole('option')
    .filter((option) => option.hasAttribute('data-highlight'));
  expect(marked.map((option) => option.getAttribute('aria-label'))).toEqual(['14:00, 60 min']);
});

it('grows from nothing only the first time a dataset appears', () => {
  const { unmount } = render(
    <RevealProvider>
      <Hourly />
    </RevealProvider>,
  );
  const firstMountBars = [...document.querySelectorAll<HTMLElement>('[role="option"] > span')];
  expect(firstMountBars.some((bar) => bar.style.transform.includes('translateY(100%)'))).toBe(true);
  unmount();
});

it('does not replay the growth when the same dataset is mounted again', () => {
  function Screen({ mounted }: { mounted: boolean }) {
    return <RevealProvider>{mounted ? <Hourly /> : null}</RevealProvider>;
  }
  vi.useFakeTimers();
  const { rerender } = render(<Screen mounted />);
  act(() => {
    vi.advanceTimersByTime(REVEAL_SETTLE_MS);
  });
  rerender(<Screen mounted={false} />);
  rerender(<Screen mounted />);

  const bars = [...document.querySelectorAll<HTMLElement>('[role="option"] > span')];
  expect(bars.some((bar) => bar.style.transform.includes('translateY(100%)'))).toBe(false);
});

it('has no accessibility violations', async () => {
  const { container } = renderWithProviders(<Hourly highlightIndex={9} />);

  await expectAccessible(container);
});
