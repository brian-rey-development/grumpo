import { describe, expect, it } from 'vitest';

import { addDays, calendarDayDifference, parseIsoDate, toIsoDate } from '@/lib/time';

describe('toIsoDate', () => {
  it('pads month and day and uses the local calendar date', () => {
    expect(toIsoDate(new Date(2026, 0, 5, 23, 59))).toBe('2026-01-05');
    expect(toIsoDate(new Date(2026, 9, 12, 0, 1))).toBe('2026-10-12');
  });

  it('round trips with parseIsoDate', () => {
    expect(toIsoDate(parseIsoDate('2026-03-08'))).toBe('2026-03-08');
  });
});

describe('addDays', () => {
  it('moves across month and year boundaries', () => {
    expect(toIsoDate(addDays(new Date(2026, 0, 1), -1))).toBe('2025-12-31');
    expect(toIsoDate(addDays(new Date(2026, 1, 27), 3))).toBe('2026-03-02');
  });

  it('keeps whole calendar days across a daylight saving change', () => {
    const start = new Date(2026, 2, 28, 12);
    expect(calendarDayDifference(addDays(start, 2), start)).toBe(2);
    expect(toIsoDate(addDays(start, 2))).toBe('2026-03-30');
  });

  it('drops the time of day', () => {
    expect(addDays(new Date(2026, 9, 12, 15, 30), 0).getHours()).toBe(0);
  });
});
