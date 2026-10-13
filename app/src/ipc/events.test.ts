import { beforeEach, expect, expectTypeOf, it, vi } from 'vitest';

const { listenMock } = vi.hoisted(() => ({ listenMock: vi.fn() }));

vi.mock('@tauri-apps/api/event', () => ({ listen: listenMock }));

import { type EventMap, subscribe } from '@/ipc/events';
import type { BlockScreen } from '@/ipc/generated/BlockScreen';
import type { FocusStatus } from '@/ipc/generated/FocusStatus';

beforeEach(() => {
  listenMock.mockReset();
});

it('types each event payload from the contract', () => {
  expectTypeOf<EventMap['usage-changed']>().toEqualTypeOf<null>();
  expectTypeOf<EventMap['status-changed']>().toEqualTypeOf<FocusStatus>();
  expectTypeOf<EventMap['block-screen']>().toEqualTypeOf<BlockScreen>();
  expectTypeOf<EventMap['block-screen-hidden']>().toEqualTypeOf<null>();
});

it('passes only the payload to the handler and returns the unlisten function', async () => {
  const unlisten = vi.fn();
  listenMock.mockImplementation(
    (_event: string, callback: (message: { payload: unknown }) => void) => {
      callback({ payload: null });
      return Promise.resolve(unlisten);
    },
  );
  const handler = vi.fn();

  const stop = await subscribe('usage-changed', handler);

  expect(listenMock).toHaveBeenCalledWith('usage-changed', expect.any(Function));
  expect(handler).toHaveBeenCalledWith(null);
  expect(stop).toBe(unlisten);
});
