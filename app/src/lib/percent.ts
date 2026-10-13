export const FULL_PERCENT = 100;

/** `part` as a percentage of `whole`, which is 0 when there is no whole to measure against. */
export function toPercent(part: number, whole: number): number {
  return whole === 0 ? 0 : (part / whole) * FULL_PERCENT;
}
