import type { Day } from '@/ipc/generated/Day';
import { cx } from '@/lib/classNames';
import { dayName, sortDays, WEEK_DAYS } from '@/lib/weekDays';

import styles from './DayPicker.module.css';

interface DayPickerProps {
  /** Accessible name of the group, such as "Days for focus time 1". */
  label: string;
  value: readonly Day[];
  onChange: (days: Day[]) => void;
  locale?: string | undefined;
  invalid?: boolean;
  /** The id of the message that explains why the group is invalid. */
  describedBy?: string | undefined;
  disabled?: boolean;
}

export function DayPicker({
  label,
  value,
  onChange,
  locale,
  invalid = false,
  describedBy,
  disabled = false,
}: DayPickerProps) {
  function toggle(day: Day): void {
    const next = value.includes(day) ? value.filter((entry) => entry !== day) : [...value, day];
    onChange(sortDays(next));
  }

  return (
    <div
      className={cx(styles.group, invalid && styles.invalid)}
      role="group"
      aria-label={label}
      aria-describedby={describedBy}
    >
      {WEEK_DAYS.map((day) => (
        <button
          key={day}
          type="button"
          className={styles.chip}
          aria-pressed={value.includes(day)}
          aria-label={dayName(day, 'long', locale)}
          disabled={disabled}
          onClick={() => {
            toggle(day);
          }}
        >
          <span aria-hidden="true">{dayName(day, 'narrow', locale)}</span>
        </button>
      ))}
    </div>
  );
}
