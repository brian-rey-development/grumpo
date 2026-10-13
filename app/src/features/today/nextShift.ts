import type { Day } from '@/ipc/generated/Day';
import type { Window } from '@/ipc/generated/Window';
import { DAYS_PER_WEEK, MINUTES_PER_HOUR } from '@/lib/time';

const DAYS_BY_INDEX: readonly Day[] = ['sun', 'mon', 'tue', 'wed', 'thu', 'fri', 'sat'];

export interface ShiftStart {
  daysAhead: number;
  /** Minutes since midnight. */
  minutes: number;
  day: Day;
}

function minutesOfDay(now: Date): number {
  return now.getHours() * MINUTES_PER_HOUR + now.getMinutes();
}

function earliestStart(windows: readonly Window[], day: Day, after: number): number | null {
  const starts = windows
    .filter((window) => window.days.includes(day) && window.start > after)
    .map((window) => window.start);
  return starts.length === 0 ? null : Math.min(...starts);
}

/** The next time a focus window opens after `now`, looking at most a week ahead. */
export function nextShiftStart(windows: readonly Window[], now: Date): ShiftStart | null {
  for (let daysAhead = 0; daysAhead <= DAYS_PER_WEEK; daysAhead += 1) {
    const day = DAYS_BY_INDEX[(now.getDay() + daysAhead) % DAYS_PER_WEEK];
    if (day === undefined) continue;
    const after = daysAhead === 0 ? minutesOfDay(now) : -1;
    const minutes = earliestStart(windows, day, after);
    if (minutes !== null) return { daysAhead, minutes, day };
  }
  return null;
}
