import { describe, expect, it } from 'vitest';

import { appItem, emptyDayReport, populatedDayReport, siteItem } from '@/test/ipc';

import { deriveTodayView } from './todayModel';

describe('deriveTodayView', () => {
  it('flags a day with nothing in it as empty', () => {
    expect(deriveTodayView(emptyDayReport('2026-10-12'), true, 9).isEmpty).toBe(true);
  });

  it('is not empty as soon as anything was tracked', () => {
    expect(deriveTodayView(populatedDayReport('2026-10-12'), true, 9).isEmpty).toBe(false);
    const onlySites = { ...emptyDayReport('2026-10-12'), topSites: [siteItem('github.com', 60)] };
    expect(deriveTodayView(onlySites, true, 9).isEmpty).toBe(false);
  });

  it('highlights the current hour only on today', () => {
    const report = populatedDayReport('2026-10-12');

    expect(deriveTodayView(report, true, 15).currentHour).toBe(15);
    expect(deriveTodayView(report, false, 15).currentHour).toBeUndefined();
  });

  it('keeps the focus windows and the day they belong to', () => {
    const view = deriveTodayView(populatedDayReport('2026-10-12'), true, 9);

    expect(view.date).toBe('2026-10-12');
    expect(view.focusWindows).toEqual([{ start: 32_400, end: 43_200 }]);
  });

  it('orders apps and sites by time spent without touching the report', () => {
    const report = {
      ...populatedDayReport('2026-10-12'),
      topApps: [appItem('a', 'A', 60), appItem('b', 'B', 600)],
      topSites: [siteItem('x.com', 10), siteItem('y.com', 20)],
    };

    const view = deriveTodayView(report, true, 9);

    expect(view.apps.map((item) => item.key)).toEqual(['b', 'a']);
    expect(view.sites.map((item) => item.key)).toEqual(['y.com', 'x.com']);
    expect(report.topApps.map((item) => item.key)).toEqual(['a', 'b']);
  });
});
