import { useFocusStatus } from '@/ipc/queries';

const NONE: readonly string[] = [];

/** The sites the service keeps blocking after it lost the saved rules, or none when it isn't reachable. */
export function useHeldSites(): readonly string[] {
  const query = useFocusStatus();
  return query.data?.kind === 'connected' ? query.data.status.heldSites : NONE;
}
