import { Minus, Plus } from 'lucide-react';

import { IconButton } from '@/components/IconButton/IconButton';
import { type StepRange, stepDown, stepUp, valueForKey } from '@/components/Stepper/stepping';

import styles from './Stepper.module.css';

const ICON_SIZE = 14;

interface StepperProps extends StepRange {
  /** Accessible name of the value, such as "Daily limit for youtube.com". */
  label: string;
  decreaseLabel: string;
  increaseLabel: string;
  value: number;
  onChange: (value: number) => void;
  formatValue: (value: number) => string;
  /** Spoken form of the value, for screen readers. Defaults to the visible text. */
  formatSpoken?: (value: number) => string;
  disabled?: boolean;
}

export function Stepper({
  label,
  decreaseLabel,
  increaseLabel,
  value,
  onChange,
  formatValue,
  formatSpoken = formatValue,
  min,
  max,
  step,
  disabled = false,
}: StepperProps) {
  const range = { min, max, step };

  return (
    <div className={styles.stepper}>
      <IconButton
        label={decreaseLabel}
        size="small"
        tabIndex={-1}
        disabled={disabled || value <= min}
        icon={<Minus size={ICON_SIZE} aria-hidden="true" />}
        onClick={() => {
          onChange(stepDown(value, range));
        }}
      />
      <div
        className={styles.value}
        role="spinbutton"
        tabIndex={disabled ? -1 : 0}
        aria-label={label}
        aria-valuemin={min}
        aria-valuemax={max}
        aria-valuenow={value}
        aria-valuetext={formatSpoken(value)}
        aria-disabled={disabled || undefined}
        onKeyDown={(event) => {
          const next = disabled ? null : valueForKey(event.key, value, range);
          if (next === null) return;
          event.preventDefault();
          if (next !== value) onChange(next);
        }}
      >
        {formatValue(value)}
      </div>
      <IconButton
        label={increaseLabel}
        size="small"
        tabIndex={-1}
        disabled={disabled || value >= max}
        icon={<Plus size={ICON_SIZE} aria-hidden="true" />}
        onClick={() => {
          onChange(stepUp(value, range));
        }}
      />
    </div>
  );
}
