import { useEffect, useEffectEvent } from 'react';

import { type EventMap, type EventName, subscribe } from '@/ipc/events';

/**
 * Subscribes to a backend event for the lifetime of the component. `onSubscribed` runs once the
 * listener is live, so callers can resync state for events emitted while it was being set up.
 */
export function useBackendEvent<K extends EventName>(
  event: K,
  onEvent: (payload: EventMap[K]) => void,
  onSubscribed?: () => void,
): void {
  const handleEvent = useEffectEvent(onEvent);
  const handleSubscribed = useEffectEvent(() => {
    onSubscribed?.();
  });

  useEffect(() => {
    let cancelled = false;
    let unsubscribe: (() => void) | null = null;

    subscribe(event, handleEvent)
      .then((stop) => {
        if (cancelled) {
          stop();
          return;
        }
        unsubscribe = stop;
        handleSubscribed();
      })
      .catch((error: unknown) => {
        console.error(`Could not subscribe to ${event}`, error);
      });

    return () => {
      cancelled = true;
      unsubscribe?.();
    };
  }, [event]);
}
