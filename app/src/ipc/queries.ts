import { keepPreviousData, useQuery, type UseQueryResult } from '@tanstack/react-query';

import { invokeCommand } from '@/ipc/commands';
import type { CommandFailure } from '@/ipc/errors';
import type { BrowserAccess } from '@/ipc/generated/BrowserAccess';
import type { ClockStyle } from '@/ipc/generated/ClockStyle';
import type { DayReport } from '@/ipc/generated/DayReport';
import type { FocusStatus } from '@/ipc/generated/FocusStatus';
import type { KnownApp } from '@/ipc/generated/KnownApp';
import type { LimitUsage } from '@/ipc/generated/LimitUsage';
import type { OnboardingState } from '@/ipc/generated/OnboardingState';
import type { Permissions } from '@/ipc/generated/Permissions';
import type { Settings } from '@/ipc/generated/Settings';
import type { WeekReport } from '@/ipc/generated/WeekReport';
import { queryKeys } from '@/ipc/queryKeys';
import type { ClockOptions } from '@/lib/format';
import { DAYS_PER_WEEK, HOURS_PER_DAY, MS_PER_HOUR, MS_PER_MINUTE } from '@/lib/time';

const APP_ICON_STALE_MS = HOURS_PER_DAY * MS_PER_HOUR;
const APP_ICON_GC_MS = DAYS_PER_WEEK * HOURS_PER_DAY * MS_PER_HOUR;
const KNOWN_APPS_STALE_MS = MS_PER_MINUTE;
const PENDING_PERMISSIONS_POLL_MS = 2000;

/** A browser that is not on this Mac, or that Grumpo reads without asking, has nothing to allow. */
export function hasAccessToAllow(entry: BrowserAccess): boolean {
  return entry.installed && entry.permission !== 'unsupported';
}

export function hasPendingPermission(permissions: Permissions): boolean {
  return (
    permissions.accessibility !== 'granted' ||
    permissions.browsers.some((entry) => hasAccessToAllow(entry) && entry.permission !== 'granted')
  );
}

export function useDayReport(date: string): UseQueryResult<DayReport, CommandFailure> {
  return useQuery({
    queryKey: queryKeys.dayReport(date),
    queryFn: () => invokeCommand('get_day_report', { date }),
    placeholderData: keepPreviousData,
  });
}

export function useWeekReport(endDate: string): UseQueryResult<WeekReport, CommandFailure> {
  return useQuery({
    queryKey: queryKeys.weekReport(endDate),
    queryFn: () => invokeCommand('get_week_report', { endDate }),
    placeholderData: keepPreviousData,
  });
}

export function useLimitUsage(): UseQueryResult<LimitUsage[], CommandFailure> {
  return useQuery({
    queryKey: queryKeys.limitUsage,
    queryFn: () => invokeCommand('get_limit_usage'),
  });
}

export function useFocusStatus(): UseQueryResult<FocusStatus, CommandFailure> {
  return useQuery({
    queryKey: queryKeys.focusStatus,
    queryFn: () => invokeCommand('get_focus_status'),
  });
}

export function useKnownApps(): UseQueryResult<KnownApp[], CommandFailure> {
  return useQuery({
    queryKey: queryKeys.knownApps,
    queryFn: () => invokeCommand('list_known_apps'),
    staleTime: KNOWN_APPS_STALE_MS,
  });
}

export function useAppIcon(bundleId: string): UseQueryResult<string | null, CommandFailure> {
  return useQuery({
    queryKey: queryKeys.appIcon(bundleId),
    queryFn: () => invokeCommand('get_app_icon', { bundleId }),
    enabled: bundleId.length > 0,
    staleTime: APP_ICON_STALE_MS,
    gcTime: APP_ICON_GC_MS,
  });
}

export function useSettings(): UseQueryResult<Settings, CommandFailure> {
  return useQuery({
    queryKey: queryKeys.settings,
    queryFn: () => invokeCommand('get_settings'),
  });
}

export function useClockStyle(): UseQueryResult<ClockStyle, CommandFailure> {
  return useQuery({
    queryKey: queryKeys.clockStyle,
    queryFn: () => invokeCommand('get_clock_style'),
    refetchOnWindowFocus: true,
  });
}

export function useClockOptions(): ClockOptions {
  return { clockStyle: useClockStyle().data };
}

/** `followPending` polls while something is not allowed yet, so a flipped switch shows up. */
export function usePermissions({ followPending = false } = {}): UseQueryResult<
  Permissions,
  CommandFailure
> {
  return useQuery({
    queryKey: queryKeys.permissions,
    queryFn: () => invokeCommand('get_permissions'),
    refetchInterval: ({ state }) =>
      followPending && state.data !== undefined && hasPendingPermission(state.data)
        ? PENDING_PERMISSIONS_POLL_MS
        : false,
  });
}

export function useOnboardingState(): UseQueryResult<OnboardingState, CommandFailure> {
  return useQuery({
    queryKey: queryKeys.onboarding,
    queryFn: () => invokeCommand('get_onboarding_state'),
  });
}
