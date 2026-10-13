import {
  SegmentedControl,
  type SegmentedOption,
} from '@/components/SegmentedControl/SegmentedControl';
import { unlockCopy } from '@/features/unlock/copy';
import { formatDuration, formatDurationLong } from '@/lib/format';
import { SECONDS_PER_MINUTE } from '@/lib/time';

import styles from './UnlockSheet.module.css';

function durationOptions(durations: readonly number[]): SegmentedOption<string>[] {
  return durations.map((minutes) => ({
    value: minutes.toString(),
    label: formatDuration(minutes * SECONDS_PER_MINUTE),
    ariaLabel: formatDurationLong(minutes * SECONDS_PER_MINUTE),
  }));
}

interface DurationChoiceProps {
  durations: readonly number[];
  minutes: number;
  onChange: (minutes: number) => void;
}

export function DurationChoice({ durations, minutes, onChange }: DurationChoiceProps) {
  return (
    <div className={styles.duration}>
      <span className={styles.durationLabel}>{unlockCopy.ready.durationLabel}</span>
      <SegmentedControl
        label={unlockCopy.ready.durationLabel}
        options={durationOptions(durations)}
        value={minutes.toString()}
        onChange={(value) => {
          onChange(Number(value));
        }}
      />
    </div>
  );
}
