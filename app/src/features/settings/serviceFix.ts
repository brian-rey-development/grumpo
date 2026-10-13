import type { DaemonUnavailableReason } from '@/ipc/generated/DaemonUnavailableReason';

export type InstallableReason = Extract<
  DaemonUnavailableReason,
  'notInstalled' | 'serviceOutdated'
>;

/** The reasons that installing fixes, since the installer also replaces an older service. */
export function offersInstall(reason: DaemonUnavailableReason): reason is InstallableReason {
  return reason === 'notInstalled' || reason === 'serviceOutdated';
}
