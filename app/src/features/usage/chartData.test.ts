import { describe, expect, it } from 'vitest';

import { hourlyChartData, weekChartData } from '@/features/usage/chartData';

const DAYS = [
  { date: '2026-10-06', totalSeconds: 3_600 },
  { date: '2026-10-07', totalSeconds: 7_200 },
  { date: '2026-10-08', totalSeconds: 0 },
  { date: '2026-10-09', totalSeconds: 1_800 },
  { date: '2026-10-10', totalSeconds: 600 },
  { date: '2026-10-11', totalSeconds: 5_400 },
  { date: '2026-10-12', totalSeconds: 15_120 },
];

describe('hourlyChartData', () => {
  it('pads short data to 24 hours and describes each hour as a range', () => {
    const data = hourlyChartData([0, 0, 0, 0, 0, 0, 0, 0, 0, 2_280]);

    expect(data.values).toHaveLength(24);
    expect(data.labelFor(9)).toBe('09:00 to 10:00');
    expect(data.labelFor(23)).toBe('23:00 to 00:00');
    expect(data.formatValue(2_280)).toBe('38min');
    expect(data.emptyText).toBe('Nothing tracked yet');
  });

  it('labels a tick at the start of each quarter day and highlights the current hour', () => {
    const data = hourlyChartData([], 14);

    expect(data.ticks.map((tick) => [tick.index, tick.label])).toEqual([
      [0, '00'],
      [6, '06'],
      [12, '12'],
      [18, '18'],
    ]);
    expect(data.highlightIndex).toBe(14);
  });
});

describe('weekChartData', () => {
  const today = new Date(2026, 9, 12, 15);

  it('names today and yesterday and the rest by date', () => {
    const data = weekChartData(DAYS, today);

    expect(data.labelFor(6)).toBe('Today');
    expect(data.labelFor(5)).toBe('Yesterday');
    expect(data.labelFor(0)).toBe('Tue 6 Oct');
    expect(data.ticks.map((tick) => tick.label)).toEqual([
      'Tue',
      'Wed',
      'Thu',
      'Fri',
      'Sat',
      'Sun',
      'Mon',
    ]);
  });

  it('highlights today by the calendar, whatever the label says', () => {
    expect(weekChartData(DAYS, today).highlightIndex).toBe(6);
    expect(weekChartData(DAYS, new Date(2026, 9, 12, 0, 0, 1)).highlightIndex).toBe(6);
  });

  it('highlights nothing when today is not among the days', () => {
    expect(weekChartData(DAYS, new Date(2026, 9, 20)).highlightIndex).toBeUndefined();
  });
});
