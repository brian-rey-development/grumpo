import { describe, expect, it } from 'vitest';

import {
  barScale,
  columnCenterPercent,
  columnStartPercent,
  sanitizeValues,
} from '@/components/ColumnChart/columns';

describe('barScale', () => {
  it('scales against the largest value and keeps a visible sliver for zero', () => {
    expect(barScale(50, 100)).toBe(0.5);
    expect(barScale(100, 100)).toBe(1);
    expect(barScale(0, 100)).toBe(0.02);
    expect(barScale(0, 0)).toBe(0.02);
  });
});

describe('column positions', () => {
  it('computes start and center percentages', () => {
    expect(columnStartPercent(6, 24)).toBe(25);
    expect(columnCenterPercent(0, 4)).toBe(12.5);
    expect(columnCenterPercent(0, 0)).toBe(0);
  });
});

describe('sanitizeValues', () => {
  it('clamps negatives and non finite numbers to zero', () => {
    expect(sanitizeValues([5, -1, Number.NaN, Number.POSITIVE_INFINITY])).toEqual([5, 0, 0, 0]);
  });
});
