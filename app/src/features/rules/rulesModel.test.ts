import { describe, expect, it } from 'vitest';

import {
  addApp,
  addLimit,
  addSite,
  addWindow,
  defaultWindow,
  hasLimitFor,
  removeApp,
  removeLimit,
  removeSite,
  removeWindow,
  RULE_CAPS,
  setLimitMinutes,
  updateWindow,
} from '@/features/rules/rulesModel';
import type { Rules } from '@/ipc/generated/Rules';
import { EMPTY_RULES, MAIL, SAMPLE_RULES, SLACK, SLACK_TARGET, YOUTUBE } from '@/test/ipc';

describe('sites', () => {
  it('adds in sorted order and ignores duplicates', () => {
    const once = addSite(addSite(EMPTY_RULES, 'reddit.com'), 'apple.com');
    expect(once.blockedSites).toEqual(['apple.com', 'reddit.com']);
    expect(addSite(once, 'apple.com')).toBe(once);
  });

  it('stops at the cap', () => {
    const full: Rules = {
      ...EMPTY_RULES,
      blockedSites: Array.from(
        { length: RULE_CAPS.sites },
        (_, index) => `site${index.toString()}.com`,
      ),
    };
    expect(addSite(full, 'one-more.com')).toBe(full);
  });

  it('removes one site and keeps the rest', () => {
    expect(removeSite(SAMPLE_RULES, 'reddit.com').blockedSites).toEqual(['youtube.com']);
  });
});

describe('apps', () => {
  it('adds by name order and ignores the same bundle id', () => {
    const rules = addApp(addApp(EMPTY_RULES, SLACK), MAIL);
    expect(rules.blockedApps.map((app) => app.name)).toEqual(['Mail', 'Slack']);
    expect(addApp(rules, { ...SLACK, name: 'Renamed' })).toBe(rules);
  });

  it('stops at the cap', () => {
    const apps = Array.from({ length: RULE_CAPS.apps }, (_, index) => ({
      bundleId: `app.${index.toString()}`,
      name: `App ${index.toString()}`,
    }));
    const full: Rules = { ...EMPTY_RULES, blockedApps: apps };
    expect(addApp(full, MAIL)).toBe(full);
  });

  it('removes by bundle id', () => {
    expect(removeApp(SAMPLE_RULES, SLACK.bundleId).blockedApps).toEqual([]);
  });
});

describe('schedule', () => {
  it('adds Monday to Friday from 09:00 to 17:00', () => {
    expect(defaultWindow()).toEqual({
      days: ['mon', 'tue', 'wed', 'thu', 'fri'],
      start: 540,
      end: 1020,
    });
    expect(addWindow(EMPTY_RULES).schedule.windows).toEqual([defaultWindow()]);
  });

  it('gives each added window its own days array', () => {
    expect(defaultWindow().days).not.toBe(defaultWindow().days);
  });

  it('stops at the cap', () => {
    const windows = Array.from({ length: RULE_CAPS.windows }, defaultWindow);
    const full: Rules = { ...EMPTY_RULES, schedule: { windows } };
    expect(addWindow(full)).toBe(full);
  });

  it('updates one window and leaves the others', () => {
    const two = addWindow(addWindow(EMPTY_RULES));
    const changed = updateWindow(two, 1, { end: 1440 });
    expect(changed.schedule.windows.map((window) => window.end)).toEqual([1020, 1440]);
  });

  it('removes a window by position', () => {
    const two = updateWindow(addWindow(addWindow(EMPTY_RULES)), 0, { start: 60 });
    expect(removeWindow(two, 0).schedule.windows).toEqual([defaultWindow()]);
  });
});

describe('limits', () => {
  it('adds a limit once per target', () => {
    const rules = addLimit(EMPTY_RULES, SLACK_TARGET, 45);
    expect(rules.limits).toEqual([{ target: SLACK_TARGET, dailyMinutes: 45 }]);
    expect(hasLimitFor(rules, { kind: 'app', app: { ...SLACK, name: 'Renamed' } })).toBe(true);
    expect(addLimit(rules, SLACK_TARGET, 10)).toBe(rules);
  });

  it('stops at the cap', () => {
    const limits = Array.from({ length: RULE_CAPS.limits }, (_, index) => ({
      target: { kind: 'site' as const, domain: `site${index.toString()}.com` },
      dailyMinutes: 30,
    }));
    const full: Rules = { ...EMPTY_RULES, limits };
    expect(addLimit(full, YOUTUBE, 30)).toBe(full);
  });

  it('changes and removes one limit', () => {
    expect(setLimitMinutes(SAMPLE_RULES, YOUTUBE, 90).limits[0]?.dailyMinutes).toBe(90);
    expect(removeLimit(SAMPLE_RULES, YOUTUBE).limits).toEqual([]);
  });

  it('does not touch other limits when changing one', () => {
    const rules = addLimit(SAMPLE_RULES, SLACK_TARGET, 20);
    const changed = setLimitMinutes(rules, SLACK_TARGET, 25);
    expect(changed.limits.map((limit) => limit.dailyMinutes)).toEqual([30, 25]);
  });
});
