import { describe, expect, it } from 'vitest';

import { type LapseState, nextLapseState, NO_LAPSE, phaseOf } from '@/features/unlock/lapse';
import type { UnlockPhaseView } from '@/ipc/generated/UnlockPhaseView';
import { connectedStatus, NOT_INSTALLED } from '@/test/ipc';

const READY: UnlockPhaseView = { kind: 'ready', readyAt: 1_000, expiresAt: 1_300 };
const LOCKED: UnlockPhaseView = { kind: 'locked' };

function after(phases: [phase: UnlockPhaseView, nowSeconds: number][]): LapseState {
  return phases.reduce(
    (state, [phase, nowSeconds]) => nextLapseState(state, phase, nowSeconds),
    NO_LAPSE,
  );
}

describe('nextLapseState', () => {
  it('remembers a ready confirmation that went back to locked after its deadline', () => {
    expect(
      after([
        [READY, 1_100],
        [LOCKED, 1_300],
      ]).lapsed,
    ).toBe(true);
    expect(
      after([
        [READY, 1_100],
        [LOCKED, 1_450],
      ]).lapsed,
    ).toBe(true);
  });

  it('does not count a confirmation cancelled before its deadline', () => {
    expect(
      after([
        [READY, 1_100],
        [LOCKED, 1_299],
      ]).lapsed,
    ).toBe(false);
  });

  it('keeps the lapse across later locked updates until a new request starts', () => {
    const lapsed = after([
      [READY, 1_100],
      [LOCKED, 1_300],
      [LOCKED, 1_400],
      [LOCKED, 1_500],
    ]);

    expect(lapsed.lapsed).toBe(true);
  });

  it.each([
    ['a new wait starts', { kind: 'coolingDown', readyAt: 2_000, minutes: 5 } as const],
    ['focus is unlocked', { kind: 'unlocked', until: 5_000 } as const],
    ['a confirmation is on show again', READY],
  ])('forgets the lapse when %s', (_, phase) => {
    const lapsed = after([
      [READY, 1_100],
      [LOCKED, 1_300],
    ]);

    expect(nextLapseState(lapsed, phase, 1_500).lapsed).toBe(false);
  });

  it('does not invent a lapse for a locked status that was never ready', () => {
    expect(
      after([
        [LOCKED, 1_000],
        [LOCKED, 2_000],
      ]).lapsed,
    ).toBe(false);
  });

  it('tracks only the confirmation that is on show', () => {
    const next = nextLapseState(NO_LAPSE, READY, 1_000);

    expect(next.readyExpiresAt).toBe(1_300);
    expect(nextLapseState(next, LOCKED, 1_300).readyExpiresAt).toBeNull();
  });
});

describe('phaseOf', () => {
  it('reads the unlock phase of a connected status only', () => {
    expect(phaseOf(connectedStatus({ phase: READY }))).toEqual(READY);
    expect(phaseOf(NOT_INSTALLED)).toBeNull();
    expect(phaseOf(undefined)).toBeNull();
  });
});
