import { describe, expect, it } from 'vitest';

import { weekReport } from '@/test/ipc';

import { deriveWeekView } from './weekModel';

function averageOverTrackedDays(days: ReturnType<typeof weekReport>['days']): number {
  return deriveWeekView({ ...weekReport('2026-10-12', [0, 0, 0, 0, 0, 0, 0]), days })
    .averageSeconds;
}

describe('the daily average', () => {
  it('averages only the days with tracked time', () => {
    const report = weekReport('2026-10-12', [600, 1_200, 0, 0, 0, 0, 1_800]);

    expect(averageOverTrackedDays(report.days)).toBe(1_200);
  });

  it('counts a full week of tracked days as seven', () => {
    const report = weekReport('2026-10-12', [60, 60, 60, 60, 60, 60, 60]);

    expect(averageOverTrackedDays(report.days)).toBe(60);
  });

  it('does not let the days before the first recorded one drag it down', () => {
    const report = weekReport('2026-10-12', [0, 0, 0, 0, 0, 3_600, 7_200]);

    expect(averageOverTrackedDays(report.days)).toBe(5_400);
  });

  it('is zero when nothing was tracked or there are no days', () => {
    expect(averageOverTrackedDays([])).toBe(0);
    expect(averageOverTrackedDays(weekReport('2026-10-12', [0, 0, 0, 0, 0, 0, 0]).days)).toBe(0);
  });
});

describe('deriveWeekView', () => {
  it('is empty only when nothing at all was tracked', () => {
    const empty = { ...weekReport('2026-10-12', [0, 0, 0, 0, 0, 0, 0]), topApps: [], topSites: [] };

    expect(deriveWeekView(empty).isEmpty).toBe(true);
    expect(deriveWeekView(weekReport('2026-10-12', [60, 0, 0, 0, 0, 0, 0])).isEmpty).toBe(false);
  });

  it('carries the average of the tracked days', () => {
    const view = deriveWeekView(weekReport('2026-10-12', [0, 1_800, 0, 3_600, 0, 0, 0]));

    expect(view.averageSeconds).toBe(2_700);
  });
});
