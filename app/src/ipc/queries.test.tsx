import { act, renderHook, waitFor } from '@testing-library/react';
import { QueryClientProvider, type UseQueryResult } from '@tanstack/react-query';
import type { ReactNode } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn<(command: string, args?: Record<string, unknown>) => Promise<unknown>>(),
}));

vi.mock('@tauri-apps/api/core', () => ({ invoke: invokeMock }));

import type { CommandFailure } from '@/ipc/errors';
import type { FocusStatus } from '@/ipc/generated/FocusStatus';
import type { Rules } from '@/ipc/generated/Rules';
import type { Settings } from '@/ipc/generated/Settings';
import {
  useAppIcon,
  useClockStyle,
  useDayReport,
  useFocusStatus,
  useKnownApps,
  useLimitUsage,
  usePermissions,
  useSettings,
  useWeekReport,
} from '@/ipc/queries';
import {
  useConfirmUnlock,
  useDeleteAllData,
  useRelock,
  useSaveRules,
  useUpdateSettings,
} from '@/ipc/mutations';
import { queryKeys } from '@/ipc/queryKeys';
import { watchWindowFocus } from '@/ipc/windowFocus';
import { createTestQueryClient } from '@/test/render';

const STATUS: FocusStatus = {
  kind: 'unavailable',
  reason: 'notInstalled',
  message: 'Grumpo service not running',
};

beforeEach(() => {
  invokeMock.mockReset();
});

function setup() {
  const client = createTestQueryClient();
  function Wrapper({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
  }
  return { client, Wrapper };
}

async function expectCommand<TData>(
  useHook: () => UseQueryResult<TData, CommandFailure>,
  command: string,
  args: Record<string, unknown> | undefined,
): Promise<void> {
  invokeMock.mockResolvedValue(null);
  const { Wrapper } = setup();

  const { result } = renderHook(useHook, { wrapper: Wrapper });

  await waitFor(() => {
    expect(result.current.isSuccess).toBe(true);
  });
  expect(invokeMock).toHaveBeenCalledWith(command, args);
}

describe('read hooks', () => {
  it('useDayReport calls get_day_report', async () => {
    await expectCommand(() => useDayReport('2026-10-12'), 'get_day_report', { date: '2026-10-12' });
  });

  it('useWeekReport calls get_week_report', async () => {
    await expectCommand(() => useWeekReport('2026-10-12'), 'get_week_report', {
      endDate: '2026-10-12',
    });
  });

  it('useLimitUsage, useFocusStatus, useKnownApps, useSettings and usePermissions take no arguments', async () => {
    await expectCommand(useLimitUsage, 'get_limit_usage', undefined);
    await expectCommand(useFocusStatus, 'get_focus_status', undefined);
    await expectCommand(useKnownApps, 'list_known_apps', undefined);
    await expectCommand(useSettings, 'get_settings', undefined);
    await expectCommand(usePermissions, 'get_permissions', undefined);
  });

  it('useAppIcon calls get_app_icon with the bundle id', async () => {
    await expectCommand(() => useAppIcon('com.apple.Safari'), 'get_app_icon', {
      bundleId: 'com.apple.Safari',
    });
  });

  it('does not ask for the icon of an empty bundle id', () => {
    const { Wrapper } = setup();

    renderHook(() => useAppIcon(''), { wrapper: Wrapper });

    expect(invokeMock).not.toHaveBeenCalled();
  });

  it('keeps the previous day on screen while the next one loads', async () => {
    invokeMock.mockResolvedValueOnce({ date: '2026-10-11' });
    const { Wrapper } = setup();
    const { result, rerender } = renderHook(({ date }) => useDayReport(date), {
      wrapper: Wrapper,
      initialProps: { date: '2026-10-11' },
    });
    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    invokeMock.mockReturnValueOnce(new Promise(() => undefined));
    rerender({ date: '2026-10-12' });

    expect(result.current.data).toEqual({ date: '2026-10-11' });
    expect(result.current.isPlaceholderData).toBe(true);
  });

  it('keeps icons fresh for a long time', async () => {
    invokeMock.mockResolvedValue('data:image/png;base64,AA==');
    const { client, Wrapper } = setup();
    const { result } = renderHook(() => useAppIcon('a'), { wrapper: Wrapper });
    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    const state = client.getQueryCache().find({ queryKey: queryKeys.appIcon('a') });

    expect(state?.isStale()).toBe(false);
    expect(state?.isStaleByTime(60 * 60_000)).toBe(false);
  });
});

describe('status mutations', () => {
  it('put the returned status in the cache', async () => {
    invokeMock.mockResolvedValue(STATUS);
    const { client, Wrapper } = setup();
    const { result } = renderHook(() => useRelock(), { wrapper: Wrapper });

    await act(async () => {
      await result.current.mutateAsync();
    });

    expect(invokeMock).toHaveBeenCalledWith('relock', undefined);
    expect(client.getQueryData(queryKeys.focusStatus)).toEqual(STATUS);
  });

  it('forward confirm_unlock arguments', async () => {
    invokeMock.mockResolvedValue(STATUS);
    const { Wrapper } = setup();
    const { result } = renderHook(() => useConfirmUnlock(), { wrapper: Wrapper });

    await act(async () => {
      await result.current.mutateAsync({ phrase: 'hello', minutes: 15 });
    });

    expect(invokeMock).toHaveBeenCalledWith('confirm_unlock', { phrase: 'hello', minutes: 15 });
  });

  it('saving rules also refreshes limit usage', async () => {
    invokeMock.mockResolvedValue(STATUS);
    const { client, Wrapper } = setup();
    const invalidate = vi.spyOn(client, 'invalidateQueries');
    const rules: Rules = {
      blockedSites: [],
      blockedApps: [],
      schedule: { windows: [] },
      limits: [],
    };
    const { result } = renderHook(() => useSaveRules(), { wrapper: Wrapper });

    await act(async () => {
      await result.current.mutateAsync(rules);
    });

    expect(invokeMock).toHaveBeenCalledWith('save_rules', { rules });
    expect(client.getQueryData(queryKeys.focusStatus)).toEqual(STATUS);
    expect(invalidate).toHaveBeenCalledWith({ queryKey: queryKeys.limitUsage });
  });
});

describe('other mutations', () => {
  it('updating settings replaces the cached settings', async () => {
    const settings: Settings = { retentionDays: 30, launchAtLogin: true };
    invokeMock.mockResolvedValue(settings);
    const { client, Wrapper } = setup();
    const { result } = renderHook(() => useUpdateSettings(), { wrapper: Wrapper });

    await act(async () => {
      await result.current.mutateAsync(settings);
    });

    expect(client.getQueryData(queryKeys.settings)).toEqual(settings);
  });

  it('deleting all data refreshes reports, limit usage and known apps', async () => {
    invokeMock.mockResolvedValue(null);
    const { client, Wrapper } = setup();
    const invalidate = vi.spyOn(client, 'invalidateQueries');
    const { result } = renderHook(() => useDeleteAllData(), { wrapper: Wrapper });

    await act(async () => {
      await result.current.mutateAsync();
    });

    expect(invalidate).toHaveBeenCalledWith({ queryKey: queryKeys.reports, refetchType: 'active' });
    expect(invalidate).toHaveBeenCalledWith({
      queryKey: queryKeys.limitUsage,
      refetchType: 'active',
    });
    expect(invalidate).toHaveBeenCalledWith({ queryKey: queryKeys.knownApps });
  });

  it('surfaces a backend failure as a CommandFailure', async () => {
    invokeMock.mockRejectedValue({ code: 'unlockNotReady', message: 'Wait for the cooldown' });
    const { Wrapper } = setup();
    const { result } = renderHook(() => useRelock(), { wrapper: Wrapper });

    act(() => {
      result.current.mutate();
    });

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });
    expect(result.current.error).toMatchObject({ code: 'unlockNotReady' });
  });
});

describe('refetching when the window gets focus', () => {
  async function mountStale<TData>(
    useHook: () => UseQueryResult<TData, CommandFailure>,
    key: readonly unknown[],
  ) {
    invokeMock.mockResolvedValue('h23');
    const { client, Wrapper } = setup();
    const { result } = renderHook(useHook, { wrapper: Wrapper });
    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });
    await client.invalidateQueries({ queryKey: key, refetchType: 'none' });
    invokeMock.mockClear();
  }

  it('refreshes the clock style when the window gets focus', async () => {
    watchWindowFocus();
    await mountStale(useClockStyle, queryKeys.clockStyle);

    act(() => {
      window.dispatchEvent(new Event('focus'));
    });

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('get_clock_style', undefined);
    });
  });

  it('leaves other queries alone, since they follow backend events instead', async () => {
    watchWindowFocus();
    await mountStale(useSettings, queryKeys.settings);

    act(() => {
      window.dispatchEvent(new Event('focus'));
    });
    await Promise.resolve();

    expect(invokeMock).not.toHaveBeenCalled();
  });
});
