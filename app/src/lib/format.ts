import type { ClockStyle } from '@/ipc/generated/ClockStyle';
import {
  calendarDayDifference,
  HOURS_PER_DAY,
  MINUTES_PER_DAY,
  pad2,
  SECONDS_PER_HOUR,
  SECONDS_PER_MINUTE,
} from '@/lib/time';

type FormatterKind = 'clock' | 'hour' | 'day' | 'weekday' | 'monthDay' | 'fullDate';

/** How clock times are written. Without a `clockStyle` the hour cycle follows the locale. */
export interface ClockOptions {
  clockStyle?: ClockStyle | undefined;
  locale?: string | undefined;
}

const DATE_OPTIONS = {
  day: { weekday: 'short', day: 'numeric', month: 'short' },
  weekday: { weekday: 'short' },
  monthDay: { day: 'numeric', month: 'short' },
  fullDate: { weekday: 'long', day: 'numeric', month: 'long', year: 'numeric' },
} as const satisfies Record<Exclude<FormatterKind, 'clock' | 'hour'>, Intl.DateTimeFormatOptions>;
const CLOCK_REFERENCE_YEAR = 2000;

const formatters = new Map<string, Intl.DateTimeFormat>();

function clockOptions(
  kind: 'clock' | 'hour',
  clockStyle: ClockStyle | undefined,
  locale: string | undefined,
): Intl.DateTimeFormatOptions {
  const minute = kind === 'clock' ? ({ minute: '2-digit' } as const) : {};
  const plain = { hour: 'numeric', ...minute } as const;
  if (clockStyle !== undefined) {
    const hour = clockStyle === 'h23' ? '2-digit' : 'numeric';
    return { ...plain, hour, hourCycle: clockStyle };
  }
  const uses24Hours = new Intl.DateTimeFormat(locale, plain).resolvedOptions().hour12 === false;
  return uses24Hours ? { ...plain, hour: '2-digit' } : plain;
}

function buildFormatter(
  kind: FormatterKind,
  { clockStyle, locale }: ClockOptions,
): Intl.DateTimeFormat {
  if (kind === 'clock' || kind === 'hour') {
    return new Intl.DateTimeFormat(locale, clockOptions(kind, clockStyle, locale));
  }
  return new Intl.DateTimeFormat(locale, DATE_OPTIONS[kind]);
}

function formatterFor(kind: FormatterKind, options: ClockOptions): Intl.DateTimeFormat {
  const cacheKey = `${kind}|${options.clockStyle ?? ''}|${options.locale ?? ''}`;
  const cached = formatters.get(cacheKey);
  if (cached) return cached;
  const created = buildFormatter(kind, options);
  formatters.set(cacheKey, created);
  return created;
}

export interface DurationParts {
  hours: number;
  minutes: number;
  seconds: number;
  underMinute: boolean;
}

export function splitDuration(seconds: number): DurationParts {
  const total = Number.isFinite(seconds) ? Math.max(0, Math.floor(seconds)) : 0;
  return {
    hours: Math.floor(total / SECONDS_PER_HOUR),
    minutes: Math.floor((total % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE),
    seconds: total % SECONDS_PER_MINUTE,
    underMinute: total > 0 && total < SECONDS_PER_MINUTE,
  };
}

export function pluralize(count: number, noun: string): string {
  return `${count.toString()} ${noun}${count === 1 ? '' : 's'}`;
}

export const DURATION_UNIT = { hour: 'h', minute: 'min' } as const;
export const UNDER_A_MINUTE = '< 1';

/** The compact form shared by every place a duration appears, such as "45min" or "1h 5min". */
export function formatDuration(seconds: number): string {
  const { hours, minutes, underMinute } = splitDuration(seconds);
  const { hour, minute } = DURATION_UNIT;
  if (underMinute) return `${UNDER_A_MINUTE}${minute}`;
  if (hours === 0) return `${minutes.toString()}${minute}`;
  if (minutes === 0) return `${hours.toString()}${hour}`;
  return `${hours.toString()}${hour} ${minutes.toString()}${minute}`;
}

/** The spelled out form for sentences and screen readers, such as "45 minutes" or "1 hour 5 minutes". */
export function formatDurationLong(seconds: number): string {
  const parts = splitDuration(seconds);
  const spoken = [
    [parts.hours, 'hour'],
    [parts.minutes, 'minute'],
    [parts.seconds, 'second'],
  ] as const;
  const present = spoken
    .filter(([count]) => count > 0)
    .map(([count, unit]) => pluralize(count, unit));
  return present.length === 0 ? pluralize(0, 'second') : present.join(' ');
}

export function formatTimeRange(start: string, end: string): string {
  return `${start} to ${end}`;
}

export function formatClock(date: Date, options: ClockOptions = {}): string {
  return formatterFor('clock', options).format(date);
}

/** A bare hour such as "6 AM" or "06", short enough to label a chart axis without wrapping. */
export function formatHourLabel(hour: number, options: ClockOptions = {}): string {
  const wrapped = ((Math.floor(hour) % HOURS_PER_DAY) + HOURS_PER_DAY) % HOURS_PER_DAY;
  return formatterFor('hour', options).format(new Date(CLOCK_REFERENCE_YEAR, 0, 1, wrapped));
}

export function formatClockFromSeconds(secondsOfDay: number, options: ClockOptions = {}): string {
  const total = Math.max(0, Math.floor(secondsOfDay));
  const hours = Math.floor(total / SECONDS_PER_HOUR) % HOURS_PER_DAY;
  const minutes = Math.floor((total % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE);
  return formatClock(new Date(CLOCK_REFERENCE_YEAR, 0, 1, hours, minutes), options);
}

/** A time of day given as minutes since midnight, where the very end of the day can get its own wording. */
export function formatTimeOfDay(
  minutes: number,
  endOfDayLabel?: string,
  options: ClockOptions = {},
): string {
  if (minutes >= MINUTES_PER_DAY && endOfDayLabel !== undefined) return endOfDayLabel;
  return formatClockFromSeconds(minutes * SECONDS_PER_MINUTE, options);
}

export function formatClockRange(
  startSeconds: number,
  endSeconds: number,
  options: ClockOptions = {},
): string {
  return formatTimeRange(
    formatClockFromSeconds(startSeconds, options),
    formatClockFromSeconds(endSeconds, options),
  );
}

/** Counts down as `mm:ss`, or `m:ss` with `padMinutes: false`, and adds an hours part when needed. */
export function formatCountdown(seconds: number, { padMinutes = true } = {}): string {
  const total = Number.isFinite(seconds) ? Math.max(0, Math.ceil(seconds)) : 0;
  const { hours, minutes } = splitDuration(total);
  const rest = total % SECONDS_PER_MINUTE;
  if (hours > 0) return `${hours.toString()}:${pad2(minutes)}:${pad2(rest)}`;
  const leading = padMinutes ? pad2(minutes) : minutes.toString();
  return `${leading}:${pad2(rest)}`;
}

export function formatDayLabel(date: Date, today: Date, locale?: string): string {
  const daysAgo = calendarDayDifference(today, date);
  if (daysAgo === 0) return 'Today';
  if (daysAgo === 1) return 'Yesterday';
  return formatterFor('day', { locale }).format(date);
}

export function formatWeekday(date: Date, locale?: string): string {
  return formatterFor('weekday', { locale }).format(date);
}

export function formatMonthDay(date: Date, locale?: string): string {
  return formatterFor('monthDay', { locale }).format(date);
}

export function formatFullDate(date: Date, locale?: string): string {
  return formatterFor('fullDate', { locale }).format(date);
}
