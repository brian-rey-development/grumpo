import { useState } from 'react';

import { Button } from '@/components/Button/Button';
import { Dialog } from '@/components/Dialog/Dialog';
import { commonCopy } from '@/copy/common';
import { describeFailure } from '@/copy/failures';
import { settingsCopy } from '@/features/settings/copy';
import { useDeleteAllData } from '@/ipc/mutations';

import styles from './Settings.module.css';

const copy = settingsCopy.deleteSheet;

/**
 * What happens to today's history. Daily limits count it, so it stays while any limit exists, and it
 * also stays when the service cannot say whether one does, because deleting it could hand time back.
 */
export type TodayPolicy = 'kept' | 'deleted' | 'keptUnknown';

const TODAY_COPY: Record<TodayPolicy, string> = {
  kept: copy.todayKept,
  deleted: copy.todayDeleted,
  keptUnknown: copy.todayKeptUnknown,
};

interface DeleteDataSheetProps {
  open: boolean;
  today: TodayPolicy;
  onClose: () => void;
  onDeleted: () => void;
}

export function DeleteDataSheet({ open, today, onClose, onDeleted }: DeleteDataSheetProps) {
  const remove = useDeleteAllData();
  const [failure, setFailure] = useState<string | null>(null);

  function confirm(): void {
    setFailure(null);
    remove.mutate(undefined, {
      onSuccess: onDeleted,
      onError: (error) => {
        setFailure(describeFailure(error, commonCopy.tryAgainSoon));
      },
    });
  }

  return (
    <Dialog
      open={open}
      title={copy.title}
      dismissible={!remove.isPending}
      onClose={onClose}
      footer={
        <>
          <Button disabled={remove.isPending} onClick={onClose}>
            {commonCopy.cancel}
          </Button>
          <Button variant="destructive" loading={remove.isPending} onClick={confirm}>
            {copy.confirm}
          </Button>
        </>
      }
    >
      <div className={styles.stack} data-selectable>
        <p>{copy.what}</p>
        <p>{copy.kept}</p>
        <p>{TODAY_COPY[today]}</p>
        <p className={styles.headline}>{copy.final}</p>
        {failure === null ? null : (
          <p className={styles.errorText} role="alert">
            {copy.failed} {failure}
          </p>
        )}
      </div>
    </Dialog>
  );
}
