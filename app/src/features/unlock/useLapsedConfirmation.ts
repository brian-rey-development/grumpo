import { hashKey, type QueryClient, useQueryClient } from '@tanstack/react-query';
import { useEffect, useState } from 'react';

import { type LapseState, nextLapseState, NO_LAPSE, phaseOf } from '@/features/unlock/lapse';
import type { FocusStatus } from '@/ipc/generated/FocusStatus';
import { queryKeys } from '@/ipc/queryKeys';
import { MS_PER_SECOND } from '@/lib/time';

const FOCUS_STATUS_HASH = hashKey(queryKeys.focusStatus);

function advance(state: LapseState, queryClient: QueryClient): LapseState {
  const phase = phaseOf(queryClient.getQueryData<FocusStatus>(queryKeys.focusStatus));
  return phase === null ? state : nextLapseState(state, phase, Date.now() / MS_PER_SECOND);
}

/**
 * True when the last confirmation ran out before it was used. It lives above the sheet, so it
 * also notices a lapse that happens while the sheet is closed, and it lasts until a new request.
 */
export function useLapsedConfirmation(): boolean {
  const queryClient = useQueryClient();
  const [lapsed, setLapsed] = useState(false);

  useEffect(() => {
    let state = advance(NO_LAPSE, queryClient);
    return queryClient.getQueryCache().subscribe((event) => {
      if (event.type !== 'updated' || event.action.type !== 'success') return;
      if (event.query.queryHash !== FOCUS_STATUS_HASH) return;
      state = advance(state, queryClient);
      setLapsed(state.lapsed);
    });
  }, [queryClient]);

  return lapsed;
}
