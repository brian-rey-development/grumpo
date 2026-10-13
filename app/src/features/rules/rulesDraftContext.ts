import { createContext, useContext } from 'react';

import type { RulesDraft } from '@/features/rules/useRulesDraft';

export const RulesDraftContext = createContext<RulesDraft | null>(null);

/** The rules being edited, which outlive the Rules screen so leaving it does not throw edits away. */
export function useRulesDraftContext(): RulesDraft {
  const value = useContext(RulesDraftContext);
  if (value === null) {
    throw new Error('useRulesDraftContext must be used inside a RulesDraftProvider');
  }
  return value;
}

/** True when there are edits waiting to be saved. */
export function useRulesDirty(): boolean {
  return useRulesDraftContext().dirty;
}
