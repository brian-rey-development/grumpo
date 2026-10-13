import { describe, expect, it } from 'vitest';

import type { BlockScreen } from '@/ipc/generated/BlockScreen';
import { formatCountdown } from '@/lib/format';

import { bodyFor, headlineFor, roundUpToMinute, secondsLeft, toneFor } from './content';
import { HEADLINES } from './copy';

function screenOf(overrides: Partial<BlockScreen>): BlockScreen {
  return {
    id: 1,
    target: { kind: 'site', domain: 'youtube.com' },
    reason: 'schedule',
    until: null,
    strict: false,
    ...overrides,
  };
}

describe('headlineFor', () => {
  it('walks through every headline in order and then starts over', () => {
    const first = HEADLINES.indexOf(headlineFor(1) as (typeof HEADLINES)[number]);
    const seen = [1, 2, 3, 4].map(headlineFor);

    expect(first).toBeGreaterThanOrEqual(0);
    expect(new Set(seen.slice(0, 3))).toEqual(new Set(HEADLINES));
    expect(seen[3]).toBe(seen[0]);
  });
});

describe('bodyFor', () => {
  it('speaks about a site that is blocked on a schedule', () => {
    expect(bodyFor(screenOf({}))).toBe(
      "You told Grumpo to block youtube.com. He's taking his job very seriously.",
    );
  });

  it('speaks about an app that was closed', () => {
    expect(bodyFor(screenOf({ target: { kind: 'app', name: 'Slack' }, reason: 'always' }))).toBe(
      'You told Grumpo to keep Slack closed. So he closed it.',
    );
  });

  it('speaks about a used up daily limit for a site or an app', () => {
    expect(bodyFor(screenOf({ reason: 'limit' }))).toBe(
      "That's all of today's youtube.com. Grumpo is guarding the rest.",
    );
    expect(bodyFor(screenOf({ target: { kind: 'app', name: 'Slack' }, reason: 'limit' }))).toBe(
      "That's all of today's Slack. Grumpo is guarding the rest.",
    );
  });
});

describe('toneFor', () => {
  it('uses the red family for a limit and the orange family for every other block', () => {
    expect(toneFor(screenOf({ reason: 'limit' }))).toBe('limit');
    expect(toneFor(screenOf({ reason: 'schedule' }))).toBe('focus');
    expect(toneFor(screenOf({ reason: 'always' }))).toBe('focus');
  });
});

describe('countdown values', () => {
  const NOW = new Date(2026, 9, 12, 15, 0, 0);
  const NOW_SECONDS = NOW.getTime() / 1000;

  it('never goes below zero once the block has ended', () => {
    expect(secondsLeft(NOW_SECONDS - 30, NOW)).toBe(0);
  });

  it.each([
    [59 * 60 + 5, '59:05'],
    [65, '1:05'],
    [5, '0:05'],
    [3_600 + 4 * 60 + 9, '1:04:09'],
    [12 * 3_600, '12:00:00'],
  ])('writes %i seconds as %s', (seconds, text) => {
    expect(formatCountdown(secondsLeft(NOW_SECONDS + seconds, NOW), { padMinutes: false })).toBe(
      text,
    );
  });

  it('rounds the spoken value up to whole minutes', () => {
    expect(roundUpToMinute(61)).toBe(120);
    expect(roundUpToMinute(120)).toBe(120);
    expect(roundUpToMinute(0)).toBe(0);
  });
});
