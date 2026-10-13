import { describe, expect, it } from 'vitest';

import type { Window } from '@/ipc/generated/Window';

import { nextShiftStart } from './nextShift';

const WEEKDAYS: Window = { days: ['mon', 'tue', 'wed', 'thu', 'fri'], start: 540, end: 1_020 };
const MONDAY_EVENING = new Date(2026, 9, 12, 19, 30);
const MONDAY_EARLY = new Date(2026, 9, 12, 7, 0);

describe('nextShiftStart', () => {
  it('finds a window that opens later today', () => {
    expect(nextShiftStart([WEEKDAYS], MONDAY_EARLY)).toEqual({
      daysAhead: 0,
      minutes: 540,
      day: 'mon',
    });
  });

  it('moves on to tomorrow once today has no window left', () => {
    expect(nextShiftStart([WEEKDAYS], MONDAY_EVENING)).toEqual({
      daysAhead: 1,
      minutes: 540,
      day: 'tue',
    });
  });

  it('skips the weekend', () => {
    const friday = new Date(2026, 9, 16, 19, 0);

    expect(nextShiftStart([WEEKDAYS], friday)).toMatchObject({ daysAhead: 3, day: 'mon' });
  });

  it('picks the earliest start when several windows open on the same day', () => {
    const late: Window = { days: ['mon'], start: 780, end: 900 };

    expect(nextShiftStart([late, WEEKDAYS], MONDAY_EARLY)?.minutes).toBe(540);
  });

  it('says nothing when no window is scheduled', () => {
    expect(nextShiftStart([], MONDAY_EVENING)).toBeNull();
  });
});
