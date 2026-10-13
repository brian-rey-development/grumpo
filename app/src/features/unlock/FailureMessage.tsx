import { commonCopy } from '@/copy/common';
import { describeFailure } from '@/copy/failures';
import type { CommandFailure } from '@/ipc/errors';

import styles from './UnlockSheet.module.css';

export function FailureMessage({ failure }: { failure: CommandFailure | null }) {
  if (failure === null) return null;

  return (
    <p className={styles.error} role="alert">
      {describeFailure(failure, commonCopy.tryAgainSoon)}
    </p>
  );
}
