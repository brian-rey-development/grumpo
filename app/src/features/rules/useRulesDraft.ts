import { useCallback, useState } from 'react';

import { rulesEqual } from '@/features/rules/rulesEqual';
import type { Rules } from '@/ipc/generated/Rules';

export interface RulesDraft {
  /** The rules on screen, which are the edited draft or else the saved rules. Null before any are known. */
  rules: Rules | null;
  /** The rules the service holds, kept from the last time it answered. */
  saved: Rules | null;
  dirty: boolean;
  /** True while the service cannot be reached, which leaves the saved rules to look at only. */
  readOnly: boolean;
  edit: (change: (rules: Rules) => Rules) => void;
  discard: () => void;
}

/**
 * A local copy of the rules to edit. With no edits the draft is the saved rules, so it follows the
 * service. Once edited it stays put until saved or discarded, even if the saved rules change.
 */
export function useRulesDraft(connected: Rules | null): RulesDraft {
  const [lastKnown, setLastKnown] = useState<Rules | null>(connected);
  const [edits, setEdits] = useState<Rules | null>(null);
  if (connected !== null && connected !== lastKnown) setLastKnown(connected);

  const saved = connected ?? lastKnown;
  const edit = useCallback(
    (change: (rules: Rules) => Rules) => {
      if (saved === null) return;
      setEdits((current) => change(current ?? saved));
    },
    [saved],
  );
  const discard = useCallback(() => {
    setEdits(null);
  }, []);

  const dirty = edits !== null && saved !== null && !rulesEqual(edits, saved);
  return { rules: edits ?? saved, saved, dirty, readOnly: connected === null, edit, discard };
}
