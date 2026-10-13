import { Button } from '@/components/Button/Button';
import { unlockCopy } from '@/features/unlock/copy';
import { FailureMessage } from '@/features/unlock/FailureMessage';
import { ReasonNote } from '@/features/unlock/ReasonNote';
import type { PhaseViewProps } from '@/features/unlock/viewProps';

import styles from './UnlockSheet.module.css';

interface LockedViewProps extends PhaseViewProps {
  cooldownMinutes: number;
  /** True when the last confirmation ran out unused, which the person deserves to be told. */
  lapsed: boolean;
}

export function LockedView({ cooldownMinutes, lapsed, actions, reason, onClose }: LockedViewProps) {
  const copy = unlockCopy.locked;

  return (
    <div className={styles.view}>
      <ReasonNote reason={reason} />
      {lapsed ? (
        <p className={styles.notice} role="status" data-selectable>
          {copy.lapsed}
        </p>
      ) : null}
      <p data-selectable>{copy.wait(cooldownMinutes)}</p>
      <p className={styles.secondary}>
        {copy.escalation} {copy.reset}
      </p>
      <FailureMessage failure={actions.failure} />
      <div className={styles.actions}>
        <Button data-autofocus disabled={actions.pending} onClick={onClose}>
          {copy.keep}
        </Button>
        <Button variant="primary" loading={actions.pending} onClick={actions.request}>
          {copy.start}
        </Button>
      </div>
    </div>
  );
}
