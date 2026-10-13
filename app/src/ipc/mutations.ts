import {
  type QueryClient,
  useMutation,
  type UseMutationResult,
  useQueryClient,
} from '@tanstack/react-query';

import { type CommandMap, invokeCommand } from '@/ipc/commands';
import type { CommandFailure } from '@/ipc/errors';
import type { FocusStatus } from '@/ipc/generated/FocusStatus';
import type { OnboardingState } from '@/ipc/generated/OnboardingState';
import type { PermissionKind } from '@/ipc/generated/PermissionKind';
import type { Permissions } from '@/ipc/generated/Permissions';
import type { Rules } from '@/ipc/generated/Rules';
import type { ServiceInstallOutcome } from '@/ipc/generated/ServiceInstallOutcome';
import type { Settings } from '@/ipc/generated/Settings';
import { queryKeys } from '@/ipc/queryKeys';

type CommandMutation<TVariables, TResult> = UseMutationResult<TResult, CommandFailure, TVariables>;

type RefetchType = 'active' | 'none';

/** With `refetchType: 'none'` the queries are only marked stale, for a window nobody is looking at. */
export async function invalidateUsage(
  queryClient: QueryClient,
  refetchType: RefetchType = 'active',
): Promise<void> {
  await Promise.all([
    queryClient.invalidateQueries({ queryKey: queryKeys.reports, refetchType }),
    queryClient.invalidateQueries({ queryKey: queryKeys.limitUsage, refetchType }),
  ]);
}

function useStatusMutation<TVariables>(
  mutationFn: (variables: TVariables) => Promise<FocusStatus>,
  afterStatus?: (queryClient: QueryClient) => Promise<void>,
  afterFailure?: (queryClient: QueryClient, failure: CommandFailure) => void,
): CommandMutation<TVariables, FocusStatus> {
  const queryClient = useQueryClient();
  return useMutation<FocusStatus, CommandFailure, TVariables>({
    mutationFn,
    onSuccess: async (status) => {
      queryClient.setQueryData(queryKeys.focusStatus, status);
      await afterStatus?.(queryClient);
    },
    onError: (failure) => {
      afterFailure?.(queryClient, failure);
    },
  });
}

export function useSaveRules(): CommandMutation<Rules, FocusStatus> {
  return useStatusMutation(
    (rules: Rules) => invokeCommand('save_rules', { rules }),
    (queryClient) => queryClient.invalidateQueries({ queryKey: queryKeys.limitUsage }),
    (queryClient, failure) => {
      if (failure.code !== 'looseningLocked') return;
      void queryClient.invalidateQueries({ queryKey: queryKeys.focusStatus });
    },
  );
}

export function useRequestUnlock(): CommandMutation<void, FocusStatus> {
  return useStatusMutation(() => invokeCommand('request_unlock'));
}

type ConfirmUnlockArgs = CommandMap['confirm_unlock']['args'];

export function useConfirmUnlock(): CommandMutation<ConfirmUnlockArgs, FocusStatus> {
  return useStatusMutation((variables: ConfirmUnlockArgs) =>
    invokeCommand('confirm_unlock', variables),
  );
}

export function useCancelUnlock(): CommandMutation<void, FocusStatus> {
  return useStatusMutation(() => invokeCommand('cancel_unlock'));
}

export function useRelock(): CommandMutation<void, FocusStatus> {
  return useStatusMutation(() => invokeCommand('relock'));
}

export function useUpdateSettings(): CommandMutation<Settings, Settings> {
  const queryClient = useQueryClient();
  return useMutation<Settings, CommandFailure, Settings>({
    mutationFn: (settings) => invokeCommand('update_settings', { settings }),
    onSuccess: (settings) => {
      queryClient.setQueryData(queryKeys.settings, settings);
    },
  });
}

export function useRequestPermission(): CommandMutation<PermissionKind, Permissions> {
  const queryClient = useQueryClient();
  return useMutation<Permissions, CommandFailure, PermissionKind>({
    mutationFn: (permission) => invokeCommand('request_permission', { permission }),
    onSuccess: (permissions) => {
      queryClient.setQueryData(queryKeys.permissions, permissions);
    },
  });
}

export function useOpenPermissionSettings(): CommandMutation<PermissionKind, null> {
  return useMutation<null, CommandFailure, PermissionKind>({
    mutationFn: (permission) => invokeCommand('open_permission_settings', { permission }),
  });
}

export function useExportData(): CommandMutation<void, string> {
  return useMutation<string, CommandFailure>({
    mutationFn: () => invokeCommand('export_data'),
  });
}

export function useDeleteAllData(): CommandMutation<void, null> {
  const queryClient = useQueryClient();
  return useMutation<null, CommandFailure>({
    mutationFn: () => invokeCommand('delete_all_data'),
    onSuccess: async () => {
      await Promise.all([
        invalidateUsage(queryClient),
        queryClient.invalidateQueries({ queryKey: queryKeys.knownApps }),
      ]);
    },
  });
}

/** A cancelled password prompt is a normal outcome, so only an install that ran refreshes the status. */
export function useInstallService(): CommandMutation<void, ServiceInstallOutcome> {
  const queryClient = useQueryClient();
  return useMutation<ServiceInstallOutcome, CommandFailure>({
    mutationFn: () => invokeCommand('install_service'),
    onSuccess: async (outcome) => {
      if (outcome !== 'installed') return;
      await queryClient.invalidateQueries({ queryKey: queryKeys.focusStatus });
    },
  });
}

function useOnboardingMutation(
  command: 'complete_onboarding' | 'reset_onboarding',
  next: OnboardingState,
): CommandMutation<void, null> {
  const queryClient = useQueryClient();
  return useMutation<null, CommandFailure>({
    mutationFn: () => invokeCommand(command),
    onSuccess: () => {
      queryClient.setQueryData(queryKeys.onboarding, next);
    },
  });
}

export function useCompleteOnboarding(): CommandMutation<void, null> {
  return useOnboardingMutation('complete_onboarding', { completed: true });
}

export function useResetOnboarding(): CommandMutation<void, null> {
  return useOnboardingMutation('reset_onboarding', { completed: false });
}
