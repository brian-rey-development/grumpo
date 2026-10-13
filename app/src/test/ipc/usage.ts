import type { DayReport } from '@/ipc/generated/DayReport';
import type { LimitUsage } from '@/ipc/generated/LimitUsage';
import type { Permissions } from '@/ipc/generated/Permissions';
import type { UsageItem } from '@/ipc/generated/UsageItem';
import type { WeekReport } from '@/ipc/generated/WeekReport';
import { addDays, parseIsoDate, SECONDS_PER_HOUR, toIsoDate } from '@/lib/time';

const HOUR = SECONDS_PER_HOUR;

export function appItem(key: string, label: string, seconds: number): UsageItem {
  return { key, label, kind: 'app', seconds };
}

export function siteItem(domain: string, seconds: number): UsageItem {
  return { key: domain, label: domain, kind: 'site', seconds };
}

export function emptyDayReport(date: string): DayReport {
  return {
    date,
    totalSeconds: 0,
    hourly: Array.from({ length: 24 }, () => 0),
    timeline: [],
    focusWindows: [],
    topApps: [],
    topSites: [],
  };
}

export function populatedDayReport(date: string): DayReport {
  const hourly = Array.from({ length: 24 }, () => 0);
  hourly[9] = 50 * 60;
  hourly[10] = 30 * 60;
  hourly[14] = 20 * 60;
  return {
    date,
    totalSeconds: 6_000,
    hourly,
    timeline: [
      { start: 9 * HOUR, end: 10 * HOUR, key: 'com.apple.Safari', label: 'Safari', kind: 'app' },
      {
        start: 10 * HOUR,
        end: 10.5 * HOUR,
        key: 'youtube.com',
        label: 'youtube.com',
        kind: 'site',
      },
    ],
    focusWindows: [{ start: 9 * HOUR, end: 12 * HOUR }],
    topApps: [
      appItem('com.apple.Safari', 'Safari', 3_600),
      appItem('com.apple.dt.Xcode', 'Xcode', 1_800),
    ],
    topSites: [siteItem('github.com', 2_400), siteItem('youtube.com', 600)],
  };
}

export function manyApps(count: number): UsageItem[] {
  return Array.from({ length: count }, (_, index) =>
    appItem(`com.example.app${index.toString()}`, `App ${index.toString()}`, (count - index) * 600),
  );
}

export function weekReport(endDate: string, dayTotals: readonly number[]): WeekReport {
  const end = parseIsoDate(endDate);
  const days = dayTotals.map((totalSeconds, index) => ({
    date: toIsoDate(addDays(end, index + 1 - dayTotals.length)),
    totalSeconds,
  }));
  return {
    days,
    totalSeconds: dayTotals.reduce((sum, value) => sum + value, 0),
    topApps: [appItem('com.apple.Safari', 'Safari', 7_200)],
    topSites: [siteItem('github.com', 3_600)],
  };
}

export function limitUsage(overrides: Partial<LimitUsage> = {}): LimitUsage {
  return {
    target: { kind: 'site', domain: 'youtube.com' },
    usedSeconds: 38 * 60,
    limitSeconds: 60 * 60,
    reached: false,
    ...overrides,
  };
}

export const GRANTED_PERMISSIONS: Permissions = {
  accessibility: 'granted',
  browsers: [{ browser: 'chrome', installed: true, permission: 'granted' }],
};

export const MISSING_BROWSER_PERMISSIONS: Permissions = {
  accessibility: 'granted',
  browsers: [
    { browser: 'chrome', installed: true, permission: 'denied' },
    { browser: 'safari', installed: true, permission: 'granted' },
  ],
};
