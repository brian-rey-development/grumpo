import type { ClockStyle } from '@/ipc/generated/ClockStyle';
import { SelectField } from '@/components/SelectField/SelectField';
import { buildTimeOptions } from '@/components/TimeField/timeOptions';
import { MINUTES_PER_DAY, TIME_STEP_MINUTES } from '@/lib/time';

const LAST_START_MINUTE = MINUTES_PER_DAY - TIME_STEP_MINUTES;

interface TimeFieldProps {
  /** Accessible name, such as "Start time". */
  label: string;
  /** Minutes since local midnight, 0 to 1440 where 1440 is the end of the day. */
  value: number;
  onChange: (minutes: number) => void;
  /** Offers 24:00 under this name, for the end of a span. Without it the day ends at 23:45. */
  endOfDayLabel?: string;
  /** The hour cycle of the labels. Without it they follow the locale. */
  clockStyle?: ClockStyle | undefined;
  stepMinutes?: number;
  min?: number;
  invalid?: boolean;
  /** The id of the message that explains why the time is invalid. */
  describedBy?: string | undefined;
  disabled?: boolean;
}

export function TimeField({
  label,
  value,
  onChange,
  endOfDayLabel,
  clockStyle,
  stepMinutes = TIME_STEP_MINUTES,
  min = 0,
  invalid = false,
  describedBy,
  disabled = false,
}: TimeFieldProps) {
  const max = endOfDayLabel === undefined ? LAST_START_MINUTE : MINUTES_PER_DAY;
  const range = { min, max, step: stepMinutes, endOfDayLabel, clockStyle };
  const options = buildTimeOptions(range, value).map((option) => ({
    value: option.minutes.toString(),
    label: option.label,
  }));

  return (
    <SelectField
      label={label}
      options={options}
      value={value.toString()}
      invalid={invalid}
      describedBy={describedBy}
      disabled={disabled}
      onChange={(next) => {
        onChange(Number(next));
      }}
    />
  );
}
