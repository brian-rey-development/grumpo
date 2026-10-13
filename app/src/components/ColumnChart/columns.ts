import { toPercent } from '@/lib/percent';

const MIN_BAR_SCALE = 0.02;
const MIN_MARKED_BAR_SCALE = 0.05;

/** A marked bar keeps enough height for its outline and fill to both show, even early in the hour. */
export function barScale(value: number, max: number, marked = false): number {
  const floor = marked ? MIN_MARKED_BAR_SCALE : MIN_BAR_SCALE;
  if (max <= 0) return floor;
  return Math.max(value / max, floor);
}

export function columnCenterPercent(index: number, count: number): number {
  return toPercent(index + 0.5, count);
}

export function columnStartPercent(index: number, count: number): number {
  return toPercent(index, count);
}

export function sanitizeValues(values: readonly number[]): number[] {
  return values.map((value) => (Number.isFinite(value) ? Math.max(value, 0) : 0));
}
