import { commonCopy } from '@/copy/common';
import type { FocusSnapshot, FocusState, UnlockWait } from '@/features/status/focusState';
import { useClockOptions } from '@/ipc/queries';
import { cx } from '@/lib/classNames';
import { formatClock, formatCountdown } from '@/lib/format';
import { MS_PER_SECOND } from '@/lib/time';
import { useNow } from '@/lib/useNow';

import { statusCopy } from './copy';

import styles from './StatusPill.module.css';

type Tone = 'focus' | 'unlocked' | 'limit' | 'neutral';

const TONES: Record<FocusState['kind'], Tone> = {
  focus: 'focus',
  unlocked: 'unlocked',
  limit: 'limit',
  free: 'neutral',
  unavailable: 'neutral',
};

function UnlockedLabel({ until }: { until: Date }) {
  const now = useNow(MS_PER_SECOND);
  const remaining = Math.max((until.getTime() - now.getTime()) / MS_PER_SECOND, 0);
  const copy = statusCopy.unlockedCountdown;
  return (
    <>
      {copy.lead}
      <span className={styles.numeric}>{formatCountdown(remaining)}</span>
      {copy.trail}
    </>
  );
}

function WaitLabel({ wait }: { wait: UnlockWait }) {
  const now = useNow(MS_PER_SECOND);
  const clock = useClockOptions();
  if (wait.kind === 'ready') {
    return <>{statusCopy.unlockReadyConfirmBy(formatClock(wait.confirmBy, clock))}</>;
  }
  const remaining = (wait.readyAt.getTime() - now.getTime()) / MS_PER_SECOND;
  if (remaining <= 0) return <>{statusCopy.unlockReady}</>;
  return (
    <>
      {statusCopy.unlockIn}
      <span className={styles.numeric}>{formatCountdown(remaining, { padMinutes: false })}</span>
    </>
  );
}

function StateLabel({ state }: { state: FocusState }) {
  const clock = useClockOptions();
  switch (state.kind) {
    case 'focus':
      return (
        <>
          {state.until
            ? statusCopy.focusOnUntil(formatClock(state.until, clock))
            : statusCopy.focusOn}
        </>
      );
    case 'unlocked':
      return <UnlockedLabel until={state.until} />;
    case 'limit':
      return <>{statusCopy.limitReached}</>;
    case 'free':
      return <>{statusCopy.free}</>;
    case 'unavailable':
      return <>{commonCopy.serviceNotRunning}</>;
  }
}

export function StatusPill({ snapshot }: { snapshot: FocusSnapshot }) {
  const { state, unlockWait } = snapshot;
  const tone = unlockWait === null ? TONES[state.kind] : 'unlocked';

  return (
    <span className={cx(styles.pill, styles[tone])}>
      <span
        className={styles.dot}
        data-hollow={state.kind === 'unavailable' || undefined}
        aria-hidden="true"
      />
      <span className={styles.label}>
        {unlockWait === null ? <StateLabel state={state} /> : <WaitLabel wait={unlockWait} />}
      </span>
    </span>
  );
}
