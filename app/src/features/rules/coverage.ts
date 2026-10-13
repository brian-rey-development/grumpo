import type { Day } from '@/ipc/generated/Day';
import type { Schedule } from '@/ipc/generated/Schedule';
import { MINUTES_PER_DAY } from '@/lib/time';
import { WEEK_DAYS } from '@/lib/weekDays';

export interface Span {
  start: number;
  end: number;
}

type DayMask = Uint8Array;

function dayMask(schedule: Schedule, day: Day): DayMask {
  const mask = new Uint8Array(MINUTES_PER_DAY);
  for (const window of schedule.windows) {
    if (!window.days.includes(day)) continue;
    mask.fill(1, Math.max(window.start, 0), Math.min(window.end, MINUTES_PER_DAY));
  }
  return mask;
}

/** The merged stretches of a day that are covered, as minutes since midnight. */
function spansOf(mask: DayMask): Span[] {
  const spans: Span[] = [];
  let start: number | null = null;
  for (let minute = 0; minute <= MINUTES_PER_DAY; minute += 1) {
    const covered = minute < MINUTES_PER_DAY && mask[minute] === 1;
    if (covered && start === null) start = minute;
    if (!covered && start !== null) {
      spans.push({ start, end: minute });
      start = null;
    }
  }
  return spans;
}

export function coverageByDay(schedule: Schedule): Record<Day, Span[]> {
  const entries = WEEK_DAYS.map((day) => [day, spansOf(dayMask(schedule, day))] as const);
  return Object.fromEntries(entries) as Record<Day, Span[]>;
}

/** Minutes a week that `from` blocks and `to` does not. */
export function minutesLost(from: Schedule, to: Schedule): number {
  return WEEK_DAYS.reduce((total, day) => {
    const before = dayMask(from, day);
    const after = dayMask(to, day);
    return (
      total +
      before.reduce(
        (sum, covered, minute) => sum + (covered === 1 && after[minute] === 0 ? 1 : 0),
        0,
      )
    );
  }, 0);
}
