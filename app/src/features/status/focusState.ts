import type { FocusStatus } from '@/ipc/generated/FocusStatus';
import { targetName } from '@/lib/limitTarget';
import { MS_PER_SECOND } from '@/lib/time';

export type FocusState =
  | { kind: 'unavailable' }
  | { kind: 'unlocked'; until: Date }
  | { kind: 'focus'; until: Date | null }
  | { kind: 'limit'; targets: string[] }
  | { kind: 'free' };

export type UnlockWait =
  { kind: 'coolingDown'; readyAt: Date } | { kind: 'ready'; confirmBy: Date };

export interface FocusSnapshot {
  state: FocusState;
  /** Set while an unlock request is cooling down or waiting for confirmation. */
  unlockWait: UnlockWait | null;
}

function fromUnixSeconds(seconds: number): Date {
  return new Date(seconds * MS_PER_SECOND);
}

function unlockWaitOf(status: FocusStatus & { kind: 'connected' }, now: Date): UnlockWait | null {
  const phase = status.status.unlock.phase;
  if (phase.kind === 'coolingDown') {
    return { kind: 'coolingDown', readyAt: fromUnixSeconds(phase.readyAt) };
  }
  if (phase.kind === 'ready' && now < fromUnixSeconds(phase.expiresAt)) {
    return { kind: 'ready', confirmBy: fromUnixSeconds(phase.expiresAt) };
  }
  return null;
}

function connectedState(status: FocusStatus & { kind: 'connected' }, now: Date): FocusState {
  const { unlock, enforcement } = status.status;
  if (unlock.phase.kind === 'unlocked' && now < fromUnixSeconds(unlock.phase.until)) {
    return { kind: 'unlocked', until: fromUnixSeconds(unlock.phase.until) };
  }
  if (!enforcement.active) return { kind: 'free' };
  if (enforcement.reason === 'limit') {
    return { kind: 'limit', targets: status.status.exhausted.map(targetName) };
  }
  const until = enforcement.activeUntil === null ? null : fromUnixSeconds(enforcement.activeUntil);
  return { kind: 'focus', until };
}

/** Collapses the daemon status into the one state the interface shows. A missing status means the service is unreachable. */
export function deriveFocus(status: FocusStatus | undefined, now: Date): FocusSnapshot {
  if (status?.kind !== 'connected') return { state: { kind: 'unavailable' }, unlockWait: null };
  const state = connectedState(status, now);
  return { state, unlockWait: state.kind === 'unlocked' ? null : unlockWaitOf(status, now) };
}

/** True when the status carries a deadline that the interface should count down to. */
export function hasLiveDeadline(status: FocusStatus | undefined): boolean {
  return status?.kind === 'connected' && status.status.unlock.phase.kind !== 'locked';
}
