import type { Day } from '@/ipc/generated/Day';

export const WEEK_DAYS: readonly Day[] = ['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun'];

const REFERENCE_MONDAY = { year: 2024, monthIndex: 0, date: 1 } as const;

type DayNameStyle = 'narrow' | 'short' | 'long';

/** Day names in the system language, built from a fixed week so the order is always Monday first. */
export function dayName(day: Day, style: DayNameStyle, locale?: string): string {
  const offset = WEEK_DAYS.indexOf(day);
  const { year, monthIndex, date } = REFERENCE_MONDAY;
  const reference = new Date(year, monthIndex, date + offset);
  return new Intl.DateTimeFormat(locale, { weekday: style }).format(reference);
}

export function sortDays(days: readonly Day[]): Day[] {
  return WEEK_DAYS.filter((day) => days.includes(day));
}
