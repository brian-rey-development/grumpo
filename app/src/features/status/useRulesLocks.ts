import { deriveFocus } from '@/features/status/focusState';
import { useFocusStatus } from '@/ipc/queries';
import { MS_PER_MINUTE } from '@/lib/time';
import { useNow } from '@/lib/useNow';

/** Which loosening changes the service refuses right now. */
export interface RulesLocks {
  /** Removing a site or app and shortening the schedule are locked while blocking is in force. */
  blocking: boolean;
  /** Raising or removing a daily limit is locked unless an unlock is active, even outside focus time. */
  limits: boolean;
}

/** Reads the status, so the locks lift as soon as an unlock lands. */
export function useRulesLocks(): RulesLocks {
  const query = useFocusStatus();
  const now = useNow(MS_PER_MINUTE);
  const { state } = deriveFocus(query.data, now);
  return {
    blocking: state.kind === 'focus' || state.kind === 'limit',
    limits: state.kind !== 'unlocked' && state.kind !== 'unavailable',
  };
}
