import { SECONDS_PER_MINUTE } from '@/lib/time';

export type Comparison = { kind: 'less' | 'more'; seconds: number } | { kind: 'same' };

function through(hourly: readonly number[], hours: number): number {
  return hourly.slice(0, hours).reduce((sum, seconds) => sum + seconds, 0);
}

/**
 * Today against yesterday over the hours that are already complete, so both sides cover the same
 * stretch of the clock. Nothing is said when yesterday has no data to compare with.
 */
export function compareThroughHour(
  today: readonly number[],
  yesterday: readonly number[],
  hour: number,
): Comparison | null {
  if (hour <= 0) return null;
  const before = through(yesterday, hour);
  if (before <= 0) return null;
  const difference = through(today, hour) - before;
  if (Math.abs(difference) < SECONDS_PER_MINUTE) return { kind: 'same' };
  return { kind: difference < 0 ? 'less' : 'more', seconds: Math.abs(difference) };
}
