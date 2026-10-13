import { Button } from '@/components/Button/Button';
import { commonCopy } from '@/copy/common';
import { statusCopy } from '@/features/status/copy';
import { unlockCopy } from '@/features/unlock/copy';
import { FailureMessage } from '@/features/unlock/FailureMessage';
import { useSecondsUntil } from '@/features/unlock/useCountdown';
import type { PhaseViewProps } from '@/features/unlock/viewProps';
import { useClockOptions } from '@/ipc/queries';
import { formatClock, formatCountdown } from '@/lib/format';
import { MS_PER_SECOND } from '@/lib/time';

import styles from './UnlockSheet.module.css';

interface UnlockedViewProps extends PhaseViewProps {
  until: number;
}

export function UnlockedView({ until, actions, reason, onClose }: UnlockedViewProps) {
  const copy = unlockCopy.unlocked;
  const remaining = useSecondsUntil(until);
  const clock = useClockOptions();

  return (
    <div className={styles.view}>
      <p className={styles.headline}>
        {statusCopy.unlockedUntil(formatClock(new Date(until * MS_PER_SECOND), clock))}
      </p>
      <p className={styles.numeric}>{copy.remaining(formatCountdown(remaining))}</p>
      <FailureMessage failure={actions.failure} />
      <div className={styles.actions}>
        <Button disabled={actions.pending} onClick={onClose}>
          {commonCopy.close}
        </Button>
        {reason.kind === 'rulesLocked' ? (
          <>
            <Button loading={actions.pending} onClick={actions.relock}>
              {copy.relock}
            </Button>
            <Button
              variant="primary"
              data-autofocus
              disabled={actions.pending}
              onClick={() => {
                reason.onUnlocked();
                onClose();
              }}
            >
              {copy.saveChanges}
            </Button>
          </>
        ) : (
          <Button
            variant="primary"
            data-autofocus
            loading={actions.pending}
            onClick={actions.relock}
          >
            {copy.relock}
          </Button>
        )}
      </div>
    </div>
  );
}
