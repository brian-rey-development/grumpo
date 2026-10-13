import { Button } from '@/components/Button/Button';
import { Skeleton, SKELETON_WIDTH } from '@/components/Skeleton/Skeleton';
import { commonCopy } from '@/copy/common';
import { describeFailure } from '@/copy/failures';
import { serviceCopy } from '@/copy/service';
import { unlockCopy } from '@/features/unlock/copy';
import type { CommandFailure } from '@/ipc/errors';
import type { DaemonUnavailableReason } from '@/ipc/generated/DaemonUnavailableReason';

import styles from './UnlockSheet.module.css';

export function LoadingView() {
  return (
    <div className={styles.view} role="status" aria-label={unlockCopy.loading}>
      <Skeleton />
      <Skeleton width={SKELETON_WIDTH.long} />
    </div>
  );
}

interface UnavailableViewProps {
  reason: DaemonUnavailableReason;
  onOpenSettings: () => void;
  onClose: () => void;
}

export function UnavailableView({ reason, onOpenSettings, onClose }: UnavailableViewProps) {
  return (
    <div className={styles.view}>
      <p data-selectable>{serviceCopy.reasons[reason]}</p>
      <div className={styles.actions}>
        <Button onClick={onClose}>{commonCopy.close}</Button>
        <Button variant="primary" data-autofocus onClick={onOpenSettings}>
          {commonCopy.openSettings}
        </Button>
      </div>
    </div>
  );
}

interface FailedViewProps {
  failure: CommandFailure;
  onRetry: () => void;
  onClose: () => void;
}

export function FailedView({ failure, onRetry, onClose }: FailedViewProps) {
  return (
    <div className={styles.view}>
      <p>{unlockCopy.failed.body}</p>
      <p className={styles.error} role="alert" data-selectable>
        {describeFailure(failure, commonCopy.tryAgainSoon)}
      </p>
      <div className={styles.actions}>
        <Button onClick={onClose}>{commonCopy.close}</Button>
        <Button variant="primary" data-autofocus onClick={onRetry}>
          {commonCopy.retry}
        </Button>
      </div>
    </div>
  );
}
