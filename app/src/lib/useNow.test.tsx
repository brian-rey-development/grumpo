import { act, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';

import { MS_PER_MINUTE, MS_PER_SECOND } from '@/lib/time';
import { useNow } from '@/lib/useNow';

function Clock({ granularity, label }: { granularity: number; label: string }) {
  const now = useNow(granularity);
  return <p data-testid={label}>{now.getTime()}</p>;
}

function setHidden(hidden: boolean): void {
  vi.spyOn(document, 'hidden', 'get').mockReturnValue(hidden);
  document.dispatchEvent(new Event('visibilitychange'));
}

beforeEach(() => {
  vi.useFakeTimers();
  vi.setSystemTime(new Date(2026, 9, 12, 12, 0, 0, 400));
});

afterEach(() => {
  vi.restoreAllMocks();
  vi.useRealTimers();
});

function shown(label: string): number {
  return Number(screen.getByTestId(label).textContent);
}

it('floors to the granularity and ticks on the clock boundary', () => {
  render(<Clock granularity={MS_PER_SECOND} label="clock" />);
  expect(shown('clock')).toBe(new Date(2026, 9, 12, 12, 0, 0, 0).getTime());

  act(() => {
    vi.advanceTimersByTime(599);
  });
  expect(shown('clock')).toBe(new Date(2026, 9, 12, 12, 0, 0, 0).getTime());

  act(() => {
    vi.advanceTimersByTime(1);
  });
  expect(shown('clock')).toBe(new Date(2026, 9, 12, 12, 0, 1, 0).getTime());
});

it('shares one timer between components that ask for the same granularity', () => {
  const setTimeoutSpy = vi.spyOn(window, 'setTimeout');

  render(
    <>
      <Clock granularity={MS_PER_MINUTE} label="a" />
      <Clock granularity={MS_PER_MINUTE} label="b" />
      <Clock granularity={MS_PER_MINUTE} label="c" />
    </>,
  );

  expect(setTimeoutSpy).toHaveBeenCalledTimes(1);
  expect(vi.getTimerCount()).toBe(1);
});

it('stops its timer when nothing listens any more', () => {
  const { unmount } = render(<Clock granularity={MS_PER_SECOND} label="clock" />);
  expect(vi.getTimerCount()).toBe(1);

  unmount();

  expect(vi.getTimerCount()).toBe(0);
});

it('sleeps while the page is hidden and catches up when it returns', () => {
  render(<Clock granularity={MS_PER_SECOND} label="clock" />);

  setHidden(true);
  expect(vi.getTimerCount()).toBe(0);
  vi.setSystemTime(new Date(2026, 9, 12, 12, 5, 0, 0));
  act(() => {
    vi.advanceTimersByTime(60_000);
  });
  expect(shown('clock')).toBe(new Date(2026, 9, 12, 12, 0, 0, 0).getTime());

  act(() => {
    setHidden(false);
  });
  expect(shown('clock')).toBe(new Date(2026, 9, 12, 12, 6, 0, 0).getTime());
  expect(vi.getTimerCount()).toBe(1);
});
