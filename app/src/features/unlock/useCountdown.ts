import { useQueryClient } from '@tanstack/react-query';
import { useEffect } from 'react';

import { queryKeys } from '@/ipc/queryKeys';
import { MS_PER_SECOND } from '@/lib/time';
import { useNow } from '@/lib/useNow';

const RETRY_INTERVAL_MS = 3 * MS_PER_SECOND;

/** Whole seconds left until a Unix timestamp in seconds, ticking once a second. */
export function useSecondsUntil(unixSeconds: number): number {
  const now = useNow(MS_PER_SECOND);
  return Math.max(0, Math.ceil(unixSeconds - now.getTime() / MS_PER_SECOND));
}

/**
 * Asks for a fresh status the moment `unixSeconds` passes, and again every few seconds while the
 * view is still on screen, because the service moves a phase forward on its own schedule.
 */
export function useRefreshWhenElapsed(unixSeconds: number): void {
  const queryClient = useQueryClient();
  const now = useNow(RETRY_INTERVAL_MS);
  const elapsed = unixSeconds * MS_PER_SECOND <= now.getTime();

  useEffect(() => {
    const untilElapsed = Math.max(0, unixSeconds * MS_PER_SECOND - Date.now());
    const timer = window.setTimeout(() => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.focusStatus });
    }, untilElapsed);
    return () => {
      window.clearTimeout(timer);
    };
  }, [unixSeconds, queryClient]);

  useEffect(() => {
    if (elapsed) void queryClient.invalidateQueries({ queryKey: queryKeys.focusStatus });
  }, [elapsed, now, queryClient]);
}
