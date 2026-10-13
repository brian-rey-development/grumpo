import { describe, expect, it } from 'vitest';

import { fileNameOf, retentionLabel } from '@/features/settings/settingsFormat';

describe('retentionLabel', () => {
  it.each([
    [30, '30 days'],
    [180, '180 days'],
    [365, '1 year'],
    [1, '1 day'],
  ])('writes %i days as %s', (days, label) => {
    expect(retentionLabel(days)).toBe(label);
  });
});

describe('fileNameOf', () => {
  it('keeps the last part of a path', () => {
    expect(fileNameOf('/Users/someone/Downloads/Focus export.json')).toBe('Focus export.json');
    expect(fileNameOf('Focus export.json')).toBe('Focus export.json');
  });
});
