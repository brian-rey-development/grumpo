import { type ReactNode, useCallback, useMemo, useState } from 'react';

import { UnlockSheet } from '@/features/unlock/UnlockSheet';
import { useLapsedConfirmation } from '@/features/unlock/useLapsedConfirmation';
import { MANUAL_REASON, type UnlockReason } from '@/features/unlock/types';
import { UnlockContext } from '@/features/unlock/unlockContext';

interface SheetState {
  open: boolean;
  reason: UnlockReason;
}

/**
 * Owns the unlock sheet. Place it inside the query, modal stack and navigation providers, and
 * wrap every screen that calls `useUnlock`.
 */
export function UnlockProvider({ children }: { children: ReactNode }) {
  const [sheet, setSheet] = useState<SheetState>({ open: false, reason: MANUAL_REASON });
  const lapsed = useLapsedConfirmation();

  const open = useCallback((reason: UnlockReason = MANUAL_REASON) => {
    setSheet({ open: true, reason });
  }, []);
  const close = useCallback(() => {
    setSheet((current) => ({ ...current, open: false }));
  }, []);
  const controls = useMemo(() => ({ open }), [open]);

  return (
    <UnlockContext value={controls}>
      {children}
      <UnlockSheet open={sheet.open} reason={sheet.reason} lapsed={lapsed} onClose={close} />
    </UnlockContext>
  );
}
