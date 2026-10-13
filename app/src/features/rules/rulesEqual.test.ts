import { expect, it } from 'vitest';

import { rulesEqual } from '@/features/rules/rulesEqual';
import { addSite, setLimitMinutes, updateWindow } from '@/features/rules/rulesModel';
import type { Rules } from '@/ipc/generated/Rules';
import { MAIL, SAMPLE_RULES, SLACK } from '@/test/ipc';

it('treats identical rules as equal', () => {
  expect(rulesEqual(SAMPLE_RULES, structuredClone(SAMPLE_RULES))).toBe(true);
});

it('ignores the order of sites, apps, days and windows', () => {
  const twoApps: Rules = { ...SAMPLE_RULES, blockedApps: [SLACK, MAIL] };
  const reordered: Rules = {
    ...twoApps,
    blockedSites: [...twoApps.blockedSites].reverse(),
    blockedApps: [MAIL, SLACK],
    schedule: {
      windows: twoApps.schedule.windows.map((window) => ({
        ...window,
        days: [...window.days].reverse(),
      })),
    },
  };
  expect(rulesEqual(twoApps, reordered)).toBe(true);
});

it('ignores a renamed app because the bundle id is its identity', () => {
  const renamed: Rules = { ...SAMPLE_RULES, blockedApps: [{ ...SLACK, name: 'Slack Desktop' }] };
  expect(rulesEqual(SAMPLE_RULES, renamed)).toBe(true);
});

it('notices every kind of real change', () => {
  const target = { kind: 'site', domain: 'youtube.com' } as const;
  expect(rulesEqual(SAMPLE_RULES, addSite(SAMPLE_RULES, 'x.com'))).toBe(false);
  expect(rulesEqual(SAMPLE_RULES, { ...SAMPLE_RULES, blockedApps: [] })).toBe(false);
  expect(rulesEqual(SAMPLE_RULES, updateWindow(SAMPLE_RULES, 0, { end: 1100 }))).toBe(false);
  expect(rulesEqual(SAMPLE_RULES, updateWindow(SAMPLE_RULES, 0, { days: ['mon'] }))).toBe(false);
  expect(rulesEqual(SAMPLE_RULES, setLimitMinutes(SAMPLE_RULES, target, 35))).toBe(false);
  expect(rulesEqual(SAMPLE_RULES, { ...SAMPLE_RULES, limits: [] })).toBe(false);
});
