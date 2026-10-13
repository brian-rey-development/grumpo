import type { ChartTick } from '@/components/ColumnChart/ChartAxis';
import type { ChartBand } from '@/components/ColumnChart/ChartBands';
import type { ColumnChartProps } from '@/components/ColumnChart/ColumnChart';
import { commonCopy } from '@/copy/common';
import type { DayTotal } from '@/ipc/generated/DayTotal';
import {
  type ClockOptions,
  formatClockRange,
  formatDayLabel,
  formatDuration,
  formatHourLabel,
  formatWeekday,
} from '@/lib/format';
import { calendarDayDifference, HOURS_PER_DAY, parseIsoDate, SECONDS_PER_HOUR } from '@/lib/time';

type ChartData = Pick<
  ColumnChartProps,
  'values' | 'labelFor' | 'formatValue' | 'ticks' | 'highlightIndex' | 'emptyText'
>;

interface FocusSpan {
  start: number;
  end: number;
}

/** Focus windows as spans of hourly columns, which is how the day chart places its shaded bands. */
export function focusBands(windows: readonly FocusSpan[]): ChartBand[] {
  return windows.map((window) => ({
    start: window.start / SECONDS_PER_HOUR,
    end: window.end / SECONDS_PER_HOUR,
  }));
}

const HOUR_TICK_STEP = 6;

function hourRange(hour: number, clock: ClockOptions): string {
  const end = ((hour + 1) % HOURS_PER_DAY) * SECONDS_PER_HOUR;
  return formatClockRange(hour * SECONDS_PER_HOUR, end, clock);
}

export function hourlyChartData(
  hourly: readonly number[],
  currentHour?: number,
  clock: ClockOptions = {},
): ChartData {
  const values = Array.from({ length: HOURS_PER_DAY }, (_, hour) => hourly[hour] ?? 0);
  const ticks: ChartTick[] = Array.from({ length: HOURS_PER_DAY / HOUR_TICK_STEP }, (_, step) => {
    const index = step * HOUR_TICK_STEP;
    return { index, label: formatHourLabel(index, clock) };
  });
  return {
    values,
    labelFor: (hour) => hourRange(hour, clock),
    formatValue: formatDuration,
    ticks,
    highlightIndex: currentHour,
    emptyText: commonCopy.nothingTracked,
  };
}

export function weekChartData(days: readonly DayTotal[], today: Date): ChartData {
  const dates = days.map((day) => parseIsoDate(day.date));
  const todayIndex = dates.findIndex((date) => calendarDayDifference(today, date) === 0);
  return {
    values: days.map((day) => day.totalSeconds),
    labelFor: (index) => {
      const date = dates[index];
      return date === undefined ? '' : formatDayLabel(date, today);
    },
    formatValue: formatDuration,
    ticks: dates.map((date, index) => ({
      index,
      label: formatWeekday(date),
      captionSeconds: days[index]?.totalSeconds ?? 0,
      align: 'center' as const,
    })),
    highlightIndex: todayIndex === -1 ? undefined : todayIndex,
    emptyText: commonCopy.nothingTracked,
  };
}
