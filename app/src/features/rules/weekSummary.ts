import { rulesCopy } from '@/features/rules/copy';
import { coverageByDay } from '@/features/rules/coverage';
import type { Schedule } from '@/ipc/generated/Schedule';
import { type ClockOptions, formatTimeOfDay, formatTimeRange } from '@/lib/format';
import { dayName, WEEK_DAYS } from '@/lib/weekDays';

const copy = rulesCopy.schedule.preview;

/** The week in words, one sentence per day, for people who cannot see the strip. */
export function summarizeWeek(schedule: Schedule, clock: ClockOptions = {}): string {
  const week = coverageByDay(schedule);
  const lines = WEEK_DAYS.map((day) => {
    const spans = week[day].map(({ start, end }) =>
      formatTimeRange(
        formatTimeOfDay(start, rulesCopy.schedule.endOfDay, clock),
        formatTimeOfDay(end, rulesCopy.schedule.endOfDay, clock),
      ),
    );
    return copy.dayLine(dayName(day, 'long'), spans.length === 0 ? copy.none : spans.join(' and '));
  });
  return copy.summary(lines.join(' '));
}
