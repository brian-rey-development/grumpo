import { Button } from '@/components/Button/Button';
import { Dialog } from '@/components/Dialog/Dialog';
import { commonCopy } from '@/copy/common';
import { settingsCopy } from '@/features/settings/copy';
import { retentionLabel } from '@/features/settings/settingsFormat';

import styles from './Settings.module.css';

const copy = settingsCopy.retentionSheet;

interface RetentionSheetProps {
  open: boolean;
  /** The shorter period that was chosen. */
  days: number;
  pending: boolean;
  failure: string | null;
  onConfirm: () => void;
  onCancel: () => void;
}

/** Shortening the period deletes history at once, so it asks first, like deleting everything does. */
export function RetentionSheet({
  open,
  days,
  pending,
  failure,
  onConfirm,
  onCancel,
}: RetentionSheetProps) {
  return (
    <Dialog
      open={open}
      title={copy.title}
      dismissible={!pending}
      onClose={onCancel}
      footer={
        <>
          <Button disabled={pending} onClick={onCancel}>
            {commonCopy.cancel}
          </Button>
          <Button variant="destructive" loading={pending} onClick={onConfirm}>
            {copy.confirm}
          </Button>
        </>
      }
    >
      <div className={styles.stack} data-selectable>
        <p>{copy.older(retentionLabel(days))}</p>
        {failure === null ? null : (
          <p className={styles.errorText} role="alert">
            {copy.failed} {failure}
          </p>
        )}
      </div>
    </Dialog>
  );
}
