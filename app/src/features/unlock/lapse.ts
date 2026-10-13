import type { FocusStatus } from '@/ipc/generated/FocusStatus';
import type { UnlockPhaseView } from '@/ipc/generated/UnlockPhaseView';

export interface LapseState {
  /** When the confirmation on show runs out, in Unix seconds, or null when none is on show. */
  readyExpiresAt: number | null;
  /** True once a confirmation ran out unused, until a new request starts. */
  lapsed: boolean;
}

export const NO_LAPSE: LapseState = { readyExpiresAt: null, lapsed: false };

/**
 * Follows the phase and notices when a ready confirmation went back to locked because its time ran
 * out, rather than because the person cancelled it, which they do before the deadline.
 */
export function nextLapseState(
  state: LapseState,
  phase: UnlockPhaseView,
  nowSeconds: number,
): LapseState {
  switch (phase.kind) {
    case 'ready':
      return { readyExpiresAt: phase.expiresAt, lapsed: false };
    case 'locked': {
      const ranOut = state.readyExpiresAt !== null && nowSeconds >= state.readyExpiresAt;
      return { readyExpiresAt: null, lapsed: state.lapsed || ranOut };
    }
    case 'coolingDown':
    case 'unlocked':
      return NO_LAPSE;
  }
}

export function phaseOf(status: FocusStatus | undefined): UnlockPhaseView | null {
  return status?.kind === 'connected' ? status.status.unlock.phase : null;
}
