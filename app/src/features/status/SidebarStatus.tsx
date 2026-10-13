import { useId } from 'react';

import { Skeleton } from '@/components/Skeleton/Skeleton';
import { useFocusStatus } from '@/ipc/queries';
import { MS_PER_MINUTE, MS_PER_SECOND } from '@/lib/time';
import { useNow } from '@/lib/useNow';

import { statusCopy } from './copy';
import { deriveFocus, hasLiveDeadline } from './focusState';
import { StatusPill } from './StatusPill';

import styles from './SidebarStatus.module.css';

const PILL_SKELETON_WIDTH = 132;

interface SidebarStatusProps {
  onOpenUnlock: () => void;
  onOpenSettings: () => void;
}

export function SidebarStatus({ onOpenUnlock, onOpenSettings }: SidebarStatusProps) {
  const query = useFocusStatus();
  const hintId = useId();
  const now = useNow(hasLiveDeadline(query.data) ? MS_PER_SECOND : MS_PER_MINUTE);

  if (query.isPending) {
    return (
      <span role="status" aria-label={statusCopy.loadingLabel}>
        <Skeleton
          width={PILL_SKELETON_WIDTH}
          height="var(--size-pill-height)"
          radius="var(--radius-full)"
        />
      </span>
    );
  }

  const snapshot = deriveFocus(query.data, now);
  const pill = <StatusPill snapshot={snapshot} />;
  if (snapshot.state.kind === 'free' && snapshot.unlockWait === null) return pill;

  const toSettings = snapshot.state.kind === 'unavailable';
  return (
    <>
      <button
        type="button"
        className={styles.button}
        aria-describedby={hintId}
        onClick={toSettings ? onOpenSettings : onOpenUnlock}
      >
        {pill}
      </button>
      <span id={hintId} className="sr-only">
        {toSettings ? statusCopy.openSettings : statusCopy.openUnlock}
      </span>
    </>
  );
}
