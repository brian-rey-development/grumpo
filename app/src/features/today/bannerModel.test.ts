import { describe, expect, it } from 'vitest';

import type { Rules } from '@/ipc/generated/Rules';
import { SAMPLE_RULES } from '@/test/ipc';

import { describeBanner } from './bannerModel';

const NOW = new Date(2026, 9, 12, 15, 30);
const CLOCK = { clockStyle: 'h23' } as const;
const BASE = { unlockWait: null, rules: SAMPLE_RULES, now: NOW, clock: CLOCK } as const;
const MINUTE_MS = 60_000;

describe('describeBanner', () => {
  it('says who is on duty until when and what is being guarded', () => {
    const banner = describeBanner({
      ...BASE,
      state: { kind: 'focus', until: new Date(2026, 9, 12, 18, 0) },
    });

    expect(banner).toMatchObject({
      tone: 'focus',
      pose: 'laptop',
      headline: 'On duty until 18:00',
      line: 'Guarding 2 sites and 1 app.',
      actions: ['editRules', 'unlock'],
    });
  });

  it('swaps the sticker and the line while an unlock cools down', () => {
    const banner = describeBanner({
      ...BASE,
      state: { kind: 'focus', until: null },
      unlockWait: { kind: 'coolingDown', readyAt: new Date(NOW.getTime() + 8 * MINUTE_MS) },
    });

    expect(banner).toMatchObject({
      pose: 'hourglass',
      headline: 'On duty',
      line: 'Unlock in 8:00.',
    });
  });

  it('names the limit that is used up and counts the limits', () => {
    const banner = describeBanner({ ...BASE, state: { kind: 'limit', targets: ['youtube.com'] } });

    expect(banner).toMatchObject({
      tone: 'limit',
      pose: 'armsCrossed',
      headline: 'youtube.com is done for today.',
      line: 'Your daily limit is used up.',
    });
  });

  it('counts the rest when several limits are used up', () => {
    const rules: Rules = {
      ...SAMPLE_RULES,
      limits: [
        ...SAMPLE_RULES.limits,
        { target: { kind: 'site', domain: 'x.com' }, dailyMinutes: 10 },
        { target: { kind: 'site', domain: 'reddit.com' }, dailyMinutes: 10 },
      ],
    };

    const banner = describeBanner({
      ...BASE,
      rules,
      state: { kind: 'limit', targets: ['youtube.com', 'x.com'] },
    });

    expect(banner.headline).toBe('youtube.com and 1 more are done for today.');
    expect(banner.line).toBe('2 of 3 daily limits used up.');
  });

  it('says how long an unlock has left', () => {
    const banner = describeBanner({
      ...BASE,
      state: { kind: 'unlocked', until: new Date(NOW.getTime() + 24 * MINUTE_MS) },
    });

    expect(banner).toMatchObject({
      tone: 'unlocked',
      pose: 'zen',
      headline: 'Unlocked until 15:54',
      line: '24min left. Grumpo is back on duty after that.',
    });
  });

  it('says when the next shift starts while off duty', () => {
    const banner = describeBanner({ ...BASE, state: { kind: 'free' } });

    expect(banner).toMatchObject({
      tone: 'off',
      pose: 'sleep',
      headline: 'Off duty',
      line: 'Next shift starts tomorrow at 09:00.',
      actions: ['editSchedule'],
    });
  });

  it('says so when nothing is scheduled', () => {
    const rules: Rules = { ...SAMPLE_RULES, schedule: { windows: [] } };

    expect(describeBanner({ ...BASE, rules, state: { kind: 'free' } }).line).toBe(
      'No focus times are scheduled.',
    );
  });

  it('points to Settings when the service is missing', () => {
    const banner = describeBanner({ ...BASE, rules: null, state: { kind: 'unavailable' } });

    expect(banner).toMatchObject({
      tone: 'unavailable',
      pose: 'stop',
      headline: 'Grumpo service not running',
      actions: ['openSettings'],
    });
  });
});
