import { cx } from '@/lib/classNames';
import {
  DURATION_UNIT,
  type DurationParts,
  formatDuration,
  splitDuration,
  UNDER_A_MINUTE,
} from '@/lib/format';

import styles from './DurationText.module.css';

interface DurationTextProps {
  seconds: number;
  className?: string | undefined;
}

function Group({ value, unit }: { value: string; unit: string }) {
  return (
    <span className={styles.group}>
      {value}
      <span className={styles.unit}>{unit}</span>
    </span>
  );
}

function DurationLayout({ parts }: { parts: DurationParts }) {
  if (parts.underMinute) return <Group value={UNDER_A_MINUTE} unit={DURATION_UNIT.minute} />;
  const showMinutes = parts.minutes > 0 || parts.hours === 0;
  return (
    <>
      {parts.hours > 0 ? <Group value={parts.hours.toString()} unit={DURATION_UNIT.hour} /> : null}
      {showMinutes ? <Group value={parts.minutes.toString()} unit={DURATION_UNIT.minute} /> : null}
    </>
  );
}

export function DurationText({ seconds, className }: DurationTextProps) {
  return (
    <span className={cx(styles.duration, className)}>
      <span aria-hidden="true">
        <DurationLayout parts={splitDuration(seconds)} />
      </span>
      <span className="sr-only">{formatDuration(seconds)}</span>
    </span>
  );
}
