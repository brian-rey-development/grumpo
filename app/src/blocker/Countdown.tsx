import type { BlockReason } from '@/ipc/generated/BlockReason';
import { formatCountdown, formatDurationLong } from '@/lib/format';
import { MS_PER_MINUTE, MS_PER_SECOND } from '@/lib/time';
import { useNow } from '@/lib/useNow';

import { blockerCopy } from './copy';
import { roundUpToMinute, secondsLeft } from './content';

import styles from './BlockScreenView.module.css';

interface CountdownProps {
  until: number;
  reason: BlockReason;
}

/** The digits are hidden from screen readers, which get a duration that only changes each minute. */
export function Countdown({ until, reason }: CountdownProps) {
  const now = useNow(MS_PER_SECOND);
  const minuteNow = useNow(MS_PER_MINUTE);
  const label = blockerCopy.countdown[reason];
  const spoken = formatDurationLong(roundUpToMinute(secondsLeft(until, minuteNow)));

  return (
    <div className={styles.countdown}>
      <p className={styles.countdownLabel}>{label}</p>
      <p className={styles.time} aria-hidden="true">
        {formatCountdown(secondsLeft(until, now), { padMinutes: false })}
      </p>
      <p className="sr-only">{spoken}</p>
    </div>
  );
}
