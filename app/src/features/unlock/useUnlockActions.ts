import { useState } from 'react';

import type { CommandFailure } from '@/ipc/errors';
import { useCancelUnlock, useConfirmUnlock, useRelock, useRequestUnlock } from '@/ipc/mutations';

export interface UnlockActions {
  request: () => void;
  cancel: () => void;
  confirm: (phrase: string, minutes: number) => void;
  relock: () => void;
  /** True while any command is in flight, which is when the sheet must not close. */
  pending: boolean;
  /**
   * The last failure. It stays through any change of phase, because the phase often moves right
   * after the click that failed, and it goes away when the person acts again or closes the sheet.
   */
  failure: CommandFailure | null;
  clear: () => void;
}

export function useUnlockActions(): UnlockActions {
  const requestMutation = useRequestUnlock();
  const cancelMutation = useCancelUnlock();
  const confirmMutation = useConfirmUnlock();
  const relockMutation = useRelock();
  const [failure, setFailure] = useState<CommandFailure | null>(null);

  const options = { onError: setFailure };

  function begin(): void {
    setFailure(null);
  }

  return {
    request: () => {
      begin();
      requestMutation.mutate(undefined, options);
    },
    cancel: () => {
      begin();
      cancelMutation.mutate(undefined, options);
    },
    confirm: (phrase, minutes) => {
      begin();
      confirmMutation.mutate({ phrase, minutes }, options);
    },
    relock: () => {
      begin();
      relockMutation.mutate(undefined, options);
    },
    pending:
      requestMutation.isPending ||
      cancelMutation.isPending ||
      confirmMutation.isPending ||
      relockMutation.isPending,
    failure,
    clear: begin,
  };
}
