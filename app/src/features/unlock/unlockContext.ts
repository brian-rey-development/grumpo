import { createContext, useContext } from 'react';

import type { UnlockControls } from '@/features/unlock/types';

export const UnlockContext = createContext<UnlockControls | null>(null);

/** Opens the unlock sheet from anywhere under an `UnlockProvider`. */
export function useUnlock(): UnlockControls {
  const value = useContext(UnlockContext);
  if (value === null) {
    throw new Error('useUnlock must be used inside an UnlockProvider');
  }
  return value;
}
