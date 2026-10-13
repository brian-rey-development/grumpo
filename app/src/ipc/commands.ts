import { invoke } from '@tauri-apps/api/core';

import { toCommandFailure } from '@/ipc/errors';
import type { BlockScreen } from '@/ipc/generated/BlockScreen';
import type { ClockStyle } from '@/ipc/generated/ClockStyle';
import type { DayReport } from '@/ipc/generated/DayReport';
import type { FocusStatus } from '@/ipc/generated/FocusStatus';
import type { KnownApp } from '@/ipc/generated/KnownApp';
import type { LimitUsage } from '@/ipc/generated/LimitUsage';
import type { OnboardingState } from '@/ipc/generated/OnboardingState';
import type { PermissionKind } from '@/ipc/generated/PermissionKind';
import type { Permissions } from '@/ipc/generated/Permissions';
import type { Rules } from '@/ipc/generated/Rules';
import type { ServiceInstallOutcome } from '@/ipc/generated/ServiceInstallOutcome';
import type { Settings } from '@/ipc/generated/Settings';
import type { WeekReport } from '@/ipc/generated/WeekReport';

/** Mirrors the "Tauri commands" table in PLAN.md. `args: null` means the command takes none. */
export interface CommandMap {
  get_day_report: { args: { date: string }; result: DayReport };
  get_week_report: { args: { endDate: string }; result: WeekReport };
  get_limit_usage: { args: null; result: LimitUsage[] };
  get_focus_status: { args: null; result: FocusStatus };
  save_rules: { args: { rules: Rules }; result: FocusStatus };
  request_unlock: { args: null; result: FocusStatus };
  confirm_unlock: { args: { phrase: string; minutes: number }; result: FocusStatus };
  cancel_unlock: { args: null; result: FocusStatus };
  relock: { args: null; result: FocusStatus };
  list_known_apps: { args: null; result: KnownApp[] };
  get_app_icon: { args: { bundleId: string }; result: string | null };
  get_settings: { args: null; result: Settings };
  get_clock_style: { args: null; result: ClockStyle };
  update_settings: { args: { settings: Settings }; result: Settings };
  get_permissions: { args: null; result: Permissions };
  request_permission: { args: { permission: PermissionKind }; result: Permissions };
  open_permission_settings: { args: { permission: PermissionKind }; result: null };
  export_data: { args: null; result: string };
  delete_all_data: { args: null; result: null };
  install_service: { args: null; result: ServiceInstallOutcome };
  get_block_screen: { args: null; result: BlockScreen | null };
  dismiss_block_screen: { args: { id?: number }; result: null };
  get_onboarding_state: { args: null; result: OnboardingState };
  complete_onboarding: { args: null; result: null };
  reset_onboarding: { args: null; result: null };
}

export type CommandName = keyof CommandMap;
export type ArgsOf<K extends CommandName> = CommandMap[K]['args'] extends null
  ? []
  : [args: CommandMap[K]['args']];
export type ResultOf<K extends CommandName> = CommandMap[K]['result'];

export async function invokeCommand<K extends CommandName>(
  command: K,
  ...args: ArgsOf<K>
): Promise<ResultOf<K>> {
  const [payload] = args as [] | [Record<string, unknown>];
  try {
    return await invoke<ResultOf<K>>(command, payload);
  } catch (rejection: unknown) {
    const failure = toCommandFailure(rejection);
    console.error(`Command ${command} failed`, failure.code, failure.message);
    throw failure;
  }
}
