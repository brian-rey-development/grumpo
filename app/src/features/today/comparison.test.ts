import { describe, expect, it } from 'vitest';

import { compareThroughHour } from './comparison';

const HOUR = 3_600;

function hours(...seconds: number[]): number[] {
  return Array.from({ length: 24 }, (_, hour) => seconds[hour] ?? 0);
}

describe('compareThroughHour', () => {
  it('says less when today is behind yesterday over the same finished hours', () => {
    const today = hours(HOUR, HOUR);
    const yesterday = hours(HOUR, 2 * HOUR);

    expect(compareThroughHour(today, yesterday, 2)).toEqual({ kind: 'less', seconds: HOUR });
  });

  it('says more when today is ahead', () => {
    expect(compareThroughHour(hours(2 * HOUR), hours(HOUR), 1)).toEqual({
      kind: 'more',
      seconds: HOUR,
    });
  });

  it('ignores the hour that is still going on and everything after it', () => {
    const today = hours(HOUR, 0, 5 * HOUR);
    const yesterday = hours(HOUR, 0, 0, 9 * HOUR);

    expect(compareThroughHour(today, yesterday, 2)).toEqual({ kind: 'same' });
  });

  it('calls a difference under a minute the same', () => {
    expect(compareThroughHour(hours(HOUR + 30), hours(HOUR), 1)).toEqual({ kind: 'same' });
  });

  it('says nothing before the first hour is over or when yesterday has no data', () => {
    expect(compareThroughHour(hours(HOUR), hours(HOUR), 0)).toBeNull();
    expect(compareThroughHour(hours(HOUR), hours(), 3)).toBeNull();
  });
});
