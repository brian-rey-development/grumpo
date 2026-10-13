export interface StepRange {
  min: number;
  max: number;
  step: number;
}

function clamp(value: number, { min, max }: StepRange): number {
  return Math.min(Math.max(value, min), max);
}

/** Moves to the next multiple of `step`, so an off-grid value snaps first and `min` and `max` stay reachable. */
export function stepUp(value: number, range: StepRange): number {
  return clamp((Math.floor(value / range.step) + 1) * range.step, range);
}

export function stepDown(value: number, range: StepRange): number {
  return clamp((Math.ceil(value / range.step) - 1) * range.step, range);
}

const KEY_STEPS: Record<string, 'up' | 'down' | 'pageUp' | 'pageDown' | 'min' | 'max'> = {
  ArrowUp: 'up',
  ArrowDown: 'down',
  PageUp: 'pageUp',
  PageDown: 'pageDown',
  Home: 'min',
  End: 'max',
};

const PAGE_STEPS = 6;

/** The value a key press leads to, or null when the key does nothing for a spin button. */
export function valueForKey(key: string, value: number, range: StepRange): number | null {
  const action = KEY_STEPS[key];
  if (action === undefined) return null;
  switch (action) {
    case 'up':
      return stepUp(value, range);
    case 'down':
      return stepDown(value, range);
    case 'pageUp':
      return clamp(value + range.step * PAGE_STEPS, range);
    case 'pageDown':
      return clamp(value - range.step * PAGE_STEPS, range);
    case 'min':
      return range.min;
    case 'max':
      return range.max;
  }
}
