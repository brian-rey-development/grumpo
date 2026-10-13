import { Stepper } from '@/components/Stepper/Stepper';
import { LIMIT_RANGE } from '@/features/rules/rulesModel';
import { formatDuration, formatDurationLong } from '@/lib/format';
import { SECONDS_PER_MINUTE } from '@/lib/time';

interface LimitStepperProps {
  label: string;
  decreaseLabel: string;
  increaseLabel: string;
  value: number;
  disabled?: boolean;
  onChange: (minutes: number) => void;
}

/** A daily limit in minutes, shown in the short form and spoken in full. */
export function LimitStepper({ value, ...rest }: LimitStepperProps) {
  return (
    <Stepper
      {...rest}
      {...LIMIT_RANGE}
      value={value}
      formatValue={(minutes) => formatDuration(minutes * SECONDS_PER_MINUTE)}
      formatSpoken={(minutes) => formatDurationLong(minutes * SECONDS_PER_MINUTE)}
    />
  );
}
