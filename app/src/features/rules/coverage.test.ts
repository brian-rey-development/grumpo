import { describe, expect, it } from 'vitest';

import { coverageByDay, minutesLost } from '@/features/rules/coverage';
import type { Schedule } from '@/ipc/generated/Schedule';

const WEEKDAYS = ['mon', 'tue', 'wed', 'thu', 'fri'] as const;

function schedule(...windows: Schedule['windows']): Schedule {
  return { windows };
}

describe('what a day covers', () => {
  it('covers start inclusive to end exclusive', () => {
    const week = coverageByDay(schedule({ days: ['mon'], start: 60, end: 120 }));
    expect(week.mon).toEqual([{ start: 60, end: 120 }]);
  });

  it('ignores windows for other days', () => {
    expect(coverageByDay(schedule({ days: ['tue'], start: 0, end: 60 })).mon).toEqual([]);
  });

  it('merges overlapping and touching windows', () => {
    const merged = schedule(
      { days: ['mon'], start: 60, end: 180 },
      { days: ['mon'], start: 120, end: 240 },
      { days: ['mon'], start: 240, end: 300 },
      { days: ['mon'], start: 600, end: 660 },
    );
    expect(coverageByDay(merged).mon).toEqual([
      { start: 60, end: 300 },
      { start: 600, end: 660 },
    ]);
  });

  it('reaches the end of the day at 1440', () => {
    const late = schedule({ days: ['sun'], start: 1380, end: 1440 });
    expect(coverageByDay(late).sun).toEqual([{ start: 1380, end: 1440 }]);
  });

  it('copes with a window that runs backwards', () => {
    expect(coverageByDay(schedule({ days: ['mon'], start: 600, end: 60 })).mon).toEqual([]);
  });
});

describe('coverageByDay', () => {
  it('lists the seven days Monday first', () => {
    const week = coverageByDay(schedule({ days: ['wed'], start: 0, end: 30 }));
    expect(Object.keys(week)).toEqual(['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun']);
    expect(week.wed).toEqual([{ start: 0, end: 30 }]);
    expect(week.thu).toEqual([]);
  });
});

describe('minutesLost', () => {
  const workWeek = schedule({ days: [...WEEKDAYS], start: 540, end: 1020 });

  it('is zero when nothing shrinks', () => {
    expect(minutesLost(workWeek, workWeek)).toBe(0);
    const longer = schedule({ days: [...WEEKDAYS], start: 480, end: 1080 });
    expect(minutesLost(workWeek, longer)).toBe(0);
  });

  it('counts the minutes dropped from the end of every day', () => {
    const shorter = schedule({ days: [...WEEKDAYS], start: 540, end: 960 });
    expect(minutesLost(workWeek, shorter)).toBe(5 * 60);
  });

  it('counts a whole removed day', () => {
    const noFriday = schedule({ days: ['mon', 'tue', 'wed', 'thu'], start: 540, end: 1020 });
    expect(minutesLost(workWeek, noFriday)).toBe(480);
  });

  it('does not count time that another window still covers', () => {
    const split = schedule(
      { days: [...WEEKDAYS], start: 540, end: 780 },
      { days: [...WEEKDAYS], start: 780, end: 1020 },
    );
    expect(minutesLost(workWeek, split)).toBe(0);
  });

  it('counts everything when the schedule is emptied', () => {
    expect(minutesLost(workWeek, schedule())).toBe(5 * 480);
  });
});
