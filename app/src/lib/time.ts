import {
  HOURS_PER_DAY,
  MINUTES_PER_DAY,
  MINUTES_PER_HOUR,
  SECONDS_PER_DAY,
  SECONDS_PER_HOUR,
  SECONDS_PER_MINUTE,
} from '@/ipc/generated/constants';

export {
  HOURS_PER_DAY,
  MINUTES_PER_DAY,
  MINUTES_PER_HOUR,
  SECONDS_PER_DAY,
  SECONDS_PER_HOUR,
  SECONDS_PER_MINUTE,
};

export const MS_PER_SECOND = 1_000;
export const MS_PER_MINUTE = SECONDS_PER_MINUTE * MS_PER_SECOND;
export const MS_PER_HOUR = SECONDS_PER_HOUR * MS_PER_SECOND;
export const DAYS_PER_WEEK = 7;
export const TIME_STEP_MINUTES = 15;

export function secondsOfDay(date: Date): number {
  return (
    date.getHours() * SECONDS_PER_HOUR + date.getMinutes() * SECONDS_PER_MINUTE + date.getSeconds()
  );
}

export function clampToDay(seconds: number): number {
  return Math.min(Math.max(seconds, 0), SECONDS_PER_DAY);
}

export function calendarDayDifference(later: Date, earlier: Date): number {
  const laterDay = Date.UTC(later.getFullYear(), later.getMonth(), later.getDate());
  const earlierDay = Date.UTC(earlier.getFullYear(), earlier.getMonth(), earlier.getDate());
  return Math.round((laterDay - earlierDay) / (SECONDS_PER_DAY * MS_PER_SECOND));
}

export function parseIsoDate(isoDate: string): Date {
  const [year = 0, month = 1, day = 1] = isoDate.split('-').map(Number);
  return new Date(year, month - 1, day);
}

export function pad2(value: number): string {
  return value.toString().padStart(2, '0');
}

/** Local calendar date as `YYYY-MM-DD`, the format the report commands take. */
export function toIsoDate(date: Date): string {
  return `${date.getFullYear().toString()}-${pad2(date.getMonth() + 1)}-${pad2(date.getDate())}`;
}

export function addDays(date: Date, days: number): Date {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate() + days);
}
