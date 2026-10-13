import { expect, it } from 'vitest';

import { hasProblems, windowProblems } from '@/features/rules/problems';
import type { Rules } from '@/ipc/generated/Rules';
import { EMPTY_RULES, SAMPLE_RULES } from '@/test/ipc';

it('finds nothing wrong with sound windows', () => {
  expect(windowProblems(SAMPLE_RULES)).toEqual([null]);
  expect(hasProblems(SAMPLE_RULES)).toBe(false);
  expect(hasProblems(EMPTY_RULES)).toBe(false);
});

it('flags windows without days or that do not move forward, per position', () => {
  const rules: Rules = {
    ...EMPTY_RULES,
    schedule: {
      windows: [
        { days: [], start: 60, end: 120 },
        { days: ['mon'], start: 120, end: 120 },
        { days: ['mon'], start: 180, end: 120 },
        { days: ['mon'], start: 0, end: 1440 },
      ],
    },
  };

  expect(windowProblems(rules)).toEqual(['noDays', 'notForward', 'notForward', null]);
  expect(hasProblems(rules)).toBe(true);
});
