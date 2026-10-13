import { sortByUsage } from '@/features/usage/usageItems';
import type { DayTotal } from '@/ipc/generated/DayTotal';
import type { UsageItem } from '@/ipc/generated/UsageItem';
import type { WeekReport } from '@/ipc/generated/WeekReport';
export interface WeekView {
  totalSeconds: number;
  /** Average over the days with tracked time, so days the Mac was off do not drag it down. */
  averageSeconds: number;
  days: DayTotal[];
  apps: UsageItem[];
  sites: UsageItem[];
  isEmpty: boolean;
}

function averageOverTrackedDays(days: readonly DayTotal[]): number {
  const tracked = days.filter((day) => day.totalSeconds > 0);
  if (tracked.length === 0) return 0;
  return tracked.reduce((sum, day) => sum + day.totalSeconds, 0) / tracked.length;
}

export function deriveWeekView(report: WeekReport): WeekView {
  return {
    totalSeconds: report.totalSeconds,
    averageSeconds: averageOverTrackedDays(report.days),
    days: report.days,
    apps: sortByUsage(report.topApps),
    sites: sortByUsage(report.topSites),
    isEmpty:
      report.totalSeconds <= 0 && report.topApps.length === 0 && report.topSites.length === 0,
  };
}
