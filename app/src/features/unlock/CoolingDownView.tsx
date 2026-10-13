import { Button } from '@/components/Button/Button';
import { commonCopy } from '@/copy/common';
import { CountdownRing } from '@/features/unlock/CountdownRing';
import { unlockCopy } from '@/features/unlock/copy';
import { FailureMessage } from '@/features/unlock/FailureMessage';
import { ReasonNote } from '@/features/unlock/ReasonNote';
import { useRefreshWhenElapsed, useSecondsUntil } from '@/features/unlock/useCountdown';
import type { PhaseViewProps } from '@/features/unlock/viewProps';
import { SECONDS_PER_MINUTE } from '@/lib/time';

import styles from './UnlockSheet.module.css';

interface CoolingDownViewProps extends PhaseViewProps {
  readyAt: number;
  /** The wait this request was given, which the service reports until the unlock is confirmed. */
  minutes: number;
}

export function CoolingDownView({
  readyAt,
  minutes,
  actions,
  reason,
  onClose,
}: CoolingDownViewProps) {
  const copy = unlockCopy.coolingDown;
  const remaining = useSecondsUntil(readyAt);
  useRefreshWhenElapsed(readyAt);

  return (
    <div className={styles.view}>
      <ReasonNote reason={reason} />
      <CountdownRing remainingSeconds={remaining} totalSeconds={minutes * SECONDS_PER_MINUTE} />
      <p className={styles.centered}>{copy.note}</p>
      <FailureMessage failure={actions.failure} />
      <div className={styles.actions}>
        <Button loading={actions.pending} onClick={actions.cancel}>
          {copy.cancel}
        </Button>
        <Button variant="primary" data-autofocus disabled={actions.pending} onClick={onClose}>
          {commonCopy.close}
        </Button>
      </div>
    </div>
  );
}
