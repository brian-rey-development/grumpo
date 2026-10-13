import { ChevronsUpDown } from 'lucide-react';

import { cx } from '@/lib/classNames';

import styles from './SelectField.module.css';

const CHEVRON_SIZE = 12;

export interface SelectOption {
  value: string;
  label: string;
}

interface SelectFieldProps {
  /** Accessible name, such as "Keep history for". */
  label: string;
  options: readonly SelectOption[];
  value: string;
  onChange: (value: string) => void;
  invalid?: boolean;
  /** The id of the message that explains why the value is invalid. */
  describedBy?: string | undefined;
  disabled?: boolean;
}

/** The system's own pop-up menu, so choosing feels like any other Mac setting and a click never commits twice. */
export function SelectField({
  label,
  options,
  value,
  onChange,
  invalid = false,
  describedBy,
  disabled = false,
}: SelectFieldProps) {
  return (
    <span className={styles.wrapper}>
      <select
        className={cx(styles.select, invalid && styles.invalid)}
        aria-label={label}
        aria-invalid={invalid || undefined}
        aria-describedby={describedBy}
        value={value}
        disabled={disabled}
        onChange={(event) => {
          onChange(event.target.value);
        }}
      >
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
      <ChevronsUpDown className={styles.chevron} size={CHEVRON_SIZE} aria-hidden="true" />
    </span>
  );
}
