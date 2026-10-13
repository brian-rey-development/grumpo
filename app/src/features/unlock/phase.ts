import type { CommandFailure } from '@/ipc/errors';
import type { DaemonUnavailableReason } from '@/ipc/generated/DaemonUnavailableReason';
import type { FocusStatus } from '@/ipc/generated/FocusStatus';
import type { UnlockView } from '@/ipc/generated/UnlockView';

export type SheetView =
  | { kind: 'loading' }
  | { kind: 'failed'; failure: CommandFailure }
  | { kind: 'unavailable'; reason: DaemonUnavailableReason }
  | { kind: 'phase'; unlock: UnlockView };

type SheetViewKey = 'loading' | 'failed' | 'unavailable' | UnlockView['phase']['kind'];

export function resolveSheetView(
  data: FocusStatus | undefined,
  error: CommandFailure | null,
): SheetView {
  if (data === undefined) {
    return error === null ? { kind: 'loading' } : { kind: 'failed', failure: error };
  }
  if (data.kind === 'unavailable') return { kind: 'unavailable', reason: data.reason };
  return { kind: 'phase', unlock: data.status.unlock };
}

export function sheetViewKey(view: SheetView): SheetViewKey {
  return view.kind === 'phase' ? view.unlock.phase.kind : view.kind;
}
