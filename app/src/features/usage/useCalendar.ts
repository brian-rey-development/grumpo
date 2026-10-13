import { useSyncExternalStore } from 'react';

import { MS_PER_MINUTE, toIsoDate } from '@/lib/time';

function subscribeToMinutes(listener: () => void): () => void {
  let timer = 0;
  const arm = () => {
    timer = window.setTimeout(
      () => {
        listener();
        arm();
      },
      MS_PER_MINUTE - (Date.now() % MS_PER_MINUTE),
    );
  };
  arm();
  return () => {
    window.clearTimeout(timer);
  };
}

/** The local hour of day. It re-renders its component only when the hour changes. */
export function useLocalHour(): number {
  return useSyncExternalStore(subscribeToMinutes, () => new Date().getHours());
}

/** Today as an ISO date. It re-renders its component only when midnight passes. */
export function useTodayIso(): string {
  return useSyncExternalStore(subscribeToMinutes, () => toIsoDate(new Date()));
}
