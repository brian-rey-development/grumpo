import { useEffect, useEffectEvent, useState } from 'react';

import { useIsModalOpen } from '@/components/Dialog/modalStack';
import { rulesCopy } from '@/features/rules/copy';
import { describeLoosening, includesLockedLimitChange } from '@/features/rules/loosening';
import { hasProblems } from '@/features/rules/problems';
import { isSaveShortcut } from '@/features/rules/saveShortcut';
import type { RulesDraft } from '@/features/rules/useRulesDraft';
import { useRulesLocks } from '@/features/status/useRulesLocks';
import { useUnlock } from '@/features/unlock/unlockContext';
import type { CommandFailure } from '@/ipc/errors';
import { useSaveRules } from '@/ipc/mutations';

export interface SaveFlow {
  /** What the button and Cmd+S do, which is to save, or to start the unlock that saving needs first. */
  submit: () => void;
  /** Saves at once, which is what the unlock sheet runs after focus has been unlocked. */
  save: () => void;
  /** Starts the unlock for the loosening changes in the draft. */
  unlock: () => void;
  /** True while the draft loosens rules that are locked, so saving has to wait for an unlock. */
  needsUnlock: boolean;
  /** True while some loosening is locked, which is whenever no unlock is active. */
  locked: boolean;
  /** True while blocking is in force, as opposed to outside focus time. */
  blocking: boolean;
  /** True while a locked change raises or removes a daily limit, which needs an unlock outside focus time too. */
  includesLimitChange: boolean;
  canSave: boolean;
  saving: boolean;
  failure: CommandFailure | null;
  clearFailure: () => void;
}

function useSaveShortcut(onSave: () => void, enabled: boolean): void {
  const handleSave = useEffectEvent(onSave);

  useEffect(() => {
    if (!enabled) return undefined;
    function onKeyDown(event: KeyboardEvent): void {
      if (!isSaveShortcut(event)) return;
      event.preventDefault();
      handleSave();
    }
    window.addEventListener('keydown', onKeyDown);
    return () => {
      window.removeEventListener('keydown', onKeyDown);
    };
  }, [enabled]);
}

/** Saves the draft and keeps the failure, so the screen can explain it while the draft stays. */
export function useSaveFlow(draft: RulesDraft): SaveFlow {
  const mutation = useSaveRules();
  const modalOpen = useIsModalOpen();
  const unlockSheet = useUnlock();
  const locks = useRulesLocks();
  const locked = locks.blocking || locks.limits;
  const [failure, setFailure] = useState<CommandFailure | null>(null);
  const { rules, saved, dirty, readOnly, discard } = draft;
  const canSave =
    dirty && !readOnly && rules !== null && !hasProblems(rules) && !mutation.isPending;
  const summary =
    dirty && saved !== null && rules !== null ? describeLoosening(saved, rules, locks) : null;
  const needsUnlock = summary !== null;
  const includesLimitChange =
    dirty && saved !== null && rules !== null && includesLockedLimitChange(saved, rules, locks);

  function save(): void {
    if (!canSave) return;
    setFailure(null);
    mutation.mutate(rules, { onSuccess: discard, onError: setFailure });
  }

  function unlock(): void {
    unlockSheet.open({
      kind: 'rulesLocked',
      summary: summary ?? rulesCopy.notices.loosening.fallbackSummary,
      onUnlocked: save,
    });
  }

  function submit(): void {
    if (!canSave) return;
    if (needsUnlock) unlock();
    else save();
  }

  useSaveShortcut(submit, canSave && !modalOpen);
  return {
    submit,
    save,
    unlock,
    needsUnlock,
    locked,
    blocking: locks.blocking,
    includesLimitChange,
    canSave,
    saving: mutation.isPending,
    failure,
    clearFailure: () => {
      setFailure(null);
    },
  };
}
