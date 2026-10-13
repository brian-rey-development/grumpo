import { unlockCopy } from '@/features/unlock/copy';
import { formatCountdown, formatDurationLong } from '@/lib/format';

import styles from './CountdownRing.module.css';

const VIEWBOX = 100;
const CENTER = VIEWBOX / 2;
const RADIUS = 46;
const STROKE = 4;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

interface CountdownRingProps {
  remainingSeconds: number;
  totalSeconds: number;
}

/**
 * The wait as an amber arc that shrinks with the time left, in the same colour and `m:ss` format as
 * the sidebar pill. The arc moves one step a second and nothing in it animates, so it reads the
 * same with reduced motion.
 */
export function CountdownRing({ remainingSeconds, totalSeconds }: CountdownRingProps) {
  const fraction = totalSeconds > 0 ? Math.min(remainingSeconds / totalSeconds, 1) : 0;

  return (
    <div className={styles.ring}>
      <svg
        className={styles.svg}
        viewBox={`0 0 ${VIEWBOX.toString()} ${VIEWBOX.toString()}`}
        aria-hidden="true"
        data-testid="countdown-arc"
      >
        <circle className={styles.track} cx={CENTER} cy={CENTER} r={RADIUS} strokeWidth={STROKE} />
        <circle
          className={styles.arc}
          cx={CENTER}
          cy={CENTER}
          r={RADIUS}
          strokeWidth={STROKE}
          strokeDasharray={CIRCUMFERENCE}
          strokeDashoffset={CIRCUMFERENCE * (1 - fraction)}
        />
      </svg>
      <p
        className={styles.time}
        role="timer"
        aria-label={`${unlockCopy.coolingDown.ringLabel}, ${formatDurationLong(remainingSeconds)}`}
      >
        {formatCountdown(remainingSeconds, { padMinutes: false })}
      </p>
    </div>
  );
}
