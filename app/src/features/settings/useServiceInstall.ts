import type { CommandFailure } from '@/ipc/errors';
import { useInstallService } from '@/ipc/mutations';

export interface ServiceInstall {
  pending: boolean;
  failure: CommandFailure | null;
  /** True in a build that has no installer, where the Terminal command is the only way. */
  unavailable: boolean;
  start: () => void;
}

/** A cancelled password prompt leaves the mutation idle with no error, so nothing here shows it. */
export function useServiceInstall(): ServiceInstall {
  const mutation = useInstallService();
  const { mutate } = mutation;

  return {
    pending: mutation.isPending,
    failure: mutation.error,
    unavailable: mutation.error?.code === 'serviceInstallUnavailable',
    start: () => {
      mutate();
    },
  };
}
