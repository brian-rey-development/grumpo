import type { ClockStyle } from '@/ipc/generated/ClockStyle';
import { formatTimeOfDay } from '@/lib/format';

interface TimeOption {
  minutes: number;
  label: string;
}

interface OptionRange {
  min: number;
  max: number;
  step: number;
  endOfDayLabel: string | undefined;
  clockStyle: ClockStyle | undefined;
}

/** Every step inside the range, plus `current` when it sits between steps so it stays selectable. */
export function buildTimeOptions(
  { min, max, step, endOfDayLabel, clockStyle }: OptionRange,
  current: number,
): TimeOption[] {
  const minutes: number[] = [];
  for (let value = min; value <= max; value += step) minutes.push(value);
  if (!minutes.includes(current)) minutes.push(current);
  return minutes
    .sort((left, right) => left - right)
    .map((value) => ({
      minutes: value,
      label: formatTimeOfDay(value, endOfDayLabel, { clockStyle }),
    }));
}
