import { useState } from 'react';

import { commonCopy } from '@/copy/common';
import { describeFailure } from '@/copy/failures';
import { settingsCopy } from '@/features/settings/copy';
import type { Settings } from '@/ipc/generated/Settings';
import { useUpdateSettings } from '@/ipc/mutations';

/**
 * Changing the period prunes history at once, so the choice is held here until it is safe. A longer
 * period deletes nothing and applies right away, and a shorter one waits for a confirmation.
 */
export function useRetentionChange(settings: Settings, onFailed: (message: string) => void) {
  const update = useUpdateSettings();
  const [choice, setChoice] = useState<number | null>(null);
  const [sheetFailure, setSheetFailure] = useState<string | null>(null);
  const saved = settings.retentionDays;
  const confirming = choice !== null && choice < saved;

  function commit(days: number): void {
    update.mutate(
      { ...settings, retentionDays: days },
      {
        onSuccess: () => {
          setChoice(null);
        },
        onError: (failure) => {
          const reason = describeFailure(failure, commonCopy.tryAgainSoon);
          if (confirming) {
            setSheetFailure(reason);
            return;
          }
          setChoice(null);
          onFailed(`${settingsCopy.history.retentionFailed} ${reason}`);
        },
      },
    );
  }

  return {
    saved,
    shown: choice ?? saved,
    confirming,
    pending: update.isPending,
    sheetFailure,
    choose: (value: string): void => {
      const days = Number(value);
      setSheetFailure(null);
      if (days === saved) {
        setChoice(null);
        return;
      }
      setChoice(days);
      if (days > saved) commit(days);
    },
    confirm: (): void => {
      if (choice !== null) commit(choice);
    },
    cancel: (): void => {
      setChoice(null);
      setSheetFailure(null);
    },
  };
}
