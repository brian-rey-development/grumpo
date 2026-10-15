import type { DaemonStatus } from '@/ipc/generated/DaemonStatus';
import type { FocusStatus } from '@/ipc/generated/FocusStatus';
import type { LimitTarget } from '@/ipc/generated/LimitTarget';
import type { Rules } from '@/ipc/generated/Rules';
import type { UnlockPhaseView } from '@/ipc/generated/UnlockPhaseView';
import { MS_PER_SECOND } from '@/lib/time';

import { SAMPLE_RULES } from './rules';

export const UNLOCK_SENTENCE = 'I am choosing to give up my focus time, and I accept that cost.';

export const LOCKED: UnlockPhaseView = { kind: 'locked' };

export function unixSeconds(date: Date): number {
  return Math.floor(date.getTime() / MS_PER_SECOND);
}

export function nowSeconds(): number {
  return unixSeconds(new Date());
}

export interface StatusOptions {
  rules?: Rules;
  phase?: UnlockPhaseView;
  nextCooldownMinutes?: number;
  /** Whether blocking is in force, which is the case unless a test says otherwise. */
  active?: boolean;
  reason?: DaemonStatus['enforcement']['reason'];
  activeUntil?: Date | null;
  exhausted?: LimitTarget[];
  /** Sites the service kept blocking after it lost the saved rules. */
  heldSites?: string[];
}

export function connectedStatus({
  rules = SAMPLE_RULES,
  phase = LOCKED,
  nextCooldownMinutes = 10,
  active = true,
  reason = active ? 'schedule' : 'none',
  activeUntil = null,
  exhausted = [],
  heldSites = [],
}: StatusOptions = {}): FocusStatus {
  return {
    kind: 'connected',
    status: {
      rules,
      enforcement: {
        active,
        reason,
        activeUntil: activeUntil === null ? null : unixSeconds(activeUntil),
        blockedSites: rules.blockedSites,
        blockedApps: rules.blockedApps,
      },
      exhausted,
      heldSites,
      unlock: { phase, nextCooldownMinutes, sentence: UNLOCK_SENTENCE, durations: [15, 30, 60] },
    },
  };
}

export const NOT_INSTALLED: FocusStatus = {
  kind: 'unavailable',
  reason: 'notInstalled',
  message: 'No socket at /var/run/grumpod.sock',
};
