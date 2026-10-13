import { unlockCopy } from '@/features/unlock/copy';
import type { UnlockReason } from '@/features/unlock/types';

import styles from './UnlockSheet.module.css';

export function ReasonNote({ reason }: { reason: UnlockReason }) {
  if (reason.kind !== 'rulesLocked') return null;

  return (
    <div className={styles.reason}>
      <p className={styles.reasonLabel}>{unlockCopy.reasonLabel}</p>
      <p data-selectable>{reason.summary}</p>
    </div>
  );
}
