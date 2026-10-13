import { describe, expect, it } from 'vitest';

import { FULL_PERCENT, toPercent } from '@/lib/percent';

describe('toPercent', () => {
  it('scales a part against its whole', () => {
    expect(toPercent(1, 4)).toBe(25);
    expect(toPercent(6, 24)).toBe(25);
    expect(toPercent(5, 5)).toBe(FULL_PERCENT);
  });

  it('is zero when there is no whole to measure against', () => {
    expect(toPercent(3, 0)).toBe(0);
  });
});
