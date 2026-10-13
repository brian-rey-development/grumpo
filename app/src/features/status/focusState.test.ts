import { describe, expect, it } from 'vitest';

import { deriveFocus, hasLiveDeadline } from '@/features/status/focusState';
import { connectedStatus, NOT_INSTALLED, unixSeconds } from '@/test/ipc';

const NOW = new Date(2026, 9, 12, 12, 0, 0);

function after(seconds: number): Date {
  return new Date(NOW.getTime() + seconds * 1_000);
}

describe('deriveFocus', () => {
  it('treats a missing or unavailable status as the service not running', () => {
    expect(deriveFocus(undefined, NOW)).toEqual({
      state: { kind: 'unavailable' },
      unlockWait: null,
    });
    expect(deriveFocus(NOT_INSTALLED, NOW).state.kind).toBe('unavailable');
  });

  it('is free when nothing is blocked and focus when something is', () => {
    expect(deriveFocus(connectedStatus({ active: false }), NOW).state).toEqual({ kind: 'free' });
    expect(
      deriveFocus(connectedStatus({ active: true, activeUntil: after(3_600) }), NOW).state,
    ).toEqual({ kind: 'focus', until: after(3_600) });
  });

  it('names the targets whose daily limit is reached', () => {
    const status = connectedStatus({
      active: true,
      reason: 'limit',
      exhausted: [{ kind: 'site', domain: 'youtube.com' }],
    });

    expect(deriveFocus(status, NOW).state).toEqual({ kind: 'limit', targets: ['youtube.com'] });
  });

  it('reports an unlock until it runs out, and no wait while unlocked', () => {
    const until = after(900);
    const live = connectedStatus({ phase: { kind: 'unlocked', until: unixSeconds(until) } });

    expect(deriveFocus(live, NOW)).toEqual({
      state: { kind: 'unlocked', until },
      unlockWait: null,
    });
    expect(deriveFocus(live, after(901)).state.kind).toBe('focus');
  });

  it('reports the wait of an unlock request that is cooling down', () => {
    const readyAt = after(252);
    const status = connectedStatus({
      phase: { kind: 'coolingDown', readyAt: unixSeconds(readyAt), minutes: 5 },
    });

    expect(deriveFocus(status, NOW).unlockWait).toEqual({ kind: 'coolingDown', readyAt });
  });

  it('reports the time to confirm by once the wait is over, and drops it when that passes', () => {
    const confirmBy = after(270);
    const status = connectedStatus({
      phase: {
        kind: 'ready',
        readyAt: unixSeconds(after(-30)),
        expiresAt: unixSeconds(confirmBy),
      },
    });

    expect(deriveFocus(status, NOW).unlockWait).toEqual({ kind: 'ready', confirmBy });
    expect(deriveFocus(status, after(271)).unlockWait).toBeNull();
  });
});

describe('hasLiveDeadline', () => {
  it('is true only while a phase counts down to something', () => {
    expect(hasLiveDeadline(undefined)).toBe(false);
    expect(hasLiveDeadline(connectedStatus())).toBe(false);
    expect(
      hasLiveDeadline(connectedStatus({ phase: { kind: 'unlocked', until: unixSeconds(NOW) } })),
    ).toBe(true);
  });
});
