export type UnlockReason =
  | { kind: 'manual' }
  | {
      kind: 'rulesLocked';
      summary: string;
      /** Runs when the person chooses to carry on once focus is unlocked, which saves the rules. */
      onUnlocked: () => void;
    };

export const MANUAL_REASON: UnlockReason = { kind: 'manual' };

export interface UnlockControls {
  open: (reason?: UnlockReason) => void;
}
