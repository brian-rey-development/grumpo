import { listen } from '@tauri-apps/api/event';

import type { BlockScreen } from '@/ipc/generated/BlockScreen';
import type { FocusStatus } from '@/ipc/generated/FocusStatus';

/** Mirrors the "Events" block in PLAN.md. */
export interface EventMap {
  'usage-changed': null;
  'status-changed': FocusStatus;
  'block-screen': BlockScreen;
  'block-screen-hidden': null;
}

export type EventName = keyof EventMap;

export async function subscribe<K extends EventName>(
  event: K,
  handler: (payload: EventMap[K]) => void,
): Promise<() => void> {
  return listen<EventMap[K]>(event, (message) => {
    handler(message.payload);
  });
}
