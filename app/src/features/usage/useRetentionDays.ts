import { useSettings } from '@/ipc/queries';

/** How many days of history Grumpo keeps, or null while the setting is not known. */
export function useRetentionDays(): number | null {
  return useSettings().data?.retentionDays ?? null;
}
