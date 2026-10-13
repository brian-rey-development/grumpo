import { sortByUsage } from '@/features/usage/usageItems';
import type { DayReport } from '@/ipc/generated/DayReport';
import type { FocusWindow } from '@/ipc/generated/FocusWindow';
import type { UsageItem } from '@/ipc/generated/UsageItem';

export interface TodayView {
  date: string;
  totalSeconds: number;
  hourly: readonly number[];
  currentHour: number | undefined;
  focusWindows: FocusWindow[];
  apps: UsageItem[];
  sites: UsageItem[];
  isEmpty: boolean;
}

function isEmptyReport(report: DayReport): boolean {
  return (
    report.totalSeconds <= 0 &&
    report.timeline.length === 0 &&
    report.topApps.length === 0 &&
    report.topSites.length === 0
  );
}

export function deriveTodayView(
  report: DayReport,
  isToday: boolean,
  currentHour: number,
): TodayView {
  return {
    date: report.date,
    totalSeconds: report.totalSeconds,
    hourly: report.hourly,
    currentHour: isToday ? currentHour : undefined,
    focusWindows: report.focusWindows.map(({ start, end }) => ({ start, end })),
    apps: sortByUsage(report.topApps),
    sites: sortByUsage(report.topSites),
    isEmpty: isEmptyReport(report),
  };
}
