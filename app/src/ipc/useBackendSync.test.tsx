import { act, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';

const { subscribeMock } = vi.hoisted(() => ({
  subscribeMock:
    vi.fn<(event: string, handler: (payload: unknown) => void) => Promise<() => void>>(),
}));

vi.mock('@/ipc/events', () => ({ subscribe: subscribeMock }));

import type { FocusStatus } from '@/ipc/generated/FocusStatus';
import { queryKeys } from '@/ipc/queryKeys';
import { useBackendSync } from '@/ipc/useBackendSync';
import { createTestQueryClient, renderWithProviders } from '@/test/render';

const UNAVAILABLE: FocusStatus = {
  kind: 'unavailable',
  reason: 'notResponding',
  message: 'Grumpo service not running',
};

const handlers = new Map<string, (payload: unknown) => void>();

beforeEach(() => {
  handlers.clear();
  subscribeMock.mockReset();
  subscribeMock.mockImplementation((event, handler) => {
    handlers.set(event, handler);
    return Promise.resolve(vi.fn());
  });
});

afterEach(() => {
  vi.restoreAllMocks();
});

function setHidden(hidden: boolean): void {
  vi.spyOn(document, 'hidden', 'get').mockReturnValue(hidden);
}

function Sync() {
  useBackendSync();
  return null;
}

async function mountSync() {
  const client = createTestQueryClient();
  const invalidate = vi.spyOn(client, 'invalidateQueries').mockResolvedValue();
  renderWithProviders(<Sync />, { client });
  await waitFor(() => {
    expect(handlers.size).toBe(2);
  });
  await waitFor(() => {
    expect(invalidate).toHaveBeenCalledTimes(3);
  });
  invalidate.mockClear();
  return { client, invalidate };
}

it('invalidates reports and limit usage when usage changes', async () => {
  const { invalidate } = await mountSync();

  handlers.get('usage-changed')?.(null);

  await waitFor(() => {
    expect(invalidate).toHaveBeenCalledWith({ queryKey: queryKeys.reports, refetchType: 'active' });
  });
  expect(invalidate).toHaveBeenCalledWith({
    queryKey: queryKeys.limitUsage,
    refetchType: 'active',
  });
});

it('only marks usage stale while the window is hidden, so nothing refetches for nobody', async () => {
  const { invalidate } = await mountSync();
  setHidden(true);

  handlers.get('usage-changed')?.(null);

  await waitFor(() => {
    expect(invalidate).toHaveBeenCalledWith({ queryKey: queryKeys.reports, refetchType: 'none' });
  });
  expect(invalidate).toHaveBeenCalledWith({ queryKey: queryKeys.limitUsage, refetchType: 'none' });
  expect(invalidate).not.toHaveBeenCalledWith(expect.objectContaining({ refetchType: 'active' }));
});

it('refetches what went stale once the window is shown again', async () => {
  const { client } = await mountSync();
  const refetch = vi.spyOn(client, 'refetchQueries').mockResolvedValue();
  setHidden(true);

  act(() => {
    document.dispatchEvent(new Event('visibilitychange'));
  });
  expect(refetch).not.toHaveBeenCalled();

  setHidden(false);
  act(() => {
    document.dispatchEvent(new Event('visibilitychange'));
  });

  expect(refetch).toHaveBeenCalledExactlyOnceWith({ type: 'active', stale: true });
});

it('stops listening for visibility when it unmounts', async () => {
  const client = createTestQueryClient();
  const refetch = vi.spyOn(client, 'refetchQueries').mockResolvedValue();
  const { unmount } = renderWithProviders(<Sync />, { client });
  await waitFor(() => {
    expect(handlers.size).toBe(2);
  });

  refetch.mockClear();
  unmount();
  document.dispatchEvent(new Event('visibilitychange'));

  expect(refetch).not.toHaveBeenCalled();
});

it('writes the status payload straight into the cache, hidden or not', async () => {
  const { client, invalidate } = await mountSync();
  setHidden(true);

  handlers.get('status-changed')?.(UNAVAILABLE);

  expect(client.getQueryData(queryKeys.focusStatus)).toEqual(UNAVAILABLE);
  expect(invalidate).not.toHaveBeenCalled();
});

it('refreshes everything once subscribed to cover events missed during setup', async () => {
  const client = createTestQueryClient();
  const invalidate = vi.spyOn(client, 'invalidateQueries').mockResolvedValue();

  renderWithProviders(<Sync />, { client });

  await waitFor(() => {
    expect(invalidate).toHaveBeenCalledWith({ queryKey: queryKeys.reports, refetchType: 'active' });
  });
  expect(invalidate).toHaveBeenCalledWith({
    queryKey: queryKeys.limitUsage,
    refetchType: 'active',
  });
  expect(invalidate).toHaveBeenCalledWith({
    queryKey: queryKeys.focusStatus,
    refetchType: 'active',
  });
});
