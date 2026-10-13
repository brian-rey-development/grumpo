import type { ReactNode } from 'react';

import { RulesDraftContext } from '@/features/rules/rulesDraftContext';
import { useRulesDraft } from '@/features/rules/useRulesDraft';
import { useFocusStatus } from '@/ipc/queries';

/**
 * Owns the draft of the rules for the whole app, so edits survive moving to another screen and
 * back. Place it inside the query provider, next to the unlock provider.
 */
export function RulesDraftProvider({ children }: { children: ReactNode }) {
  const query = useFocusStatus();
  const connected = query.data?.kind === 'connected' ? query.data.status.rules : null;
  const draft = useRulesDraft(connected);

  return <RulesDraftContext value={draft}>{children}</RulesDraftContext>;
}
