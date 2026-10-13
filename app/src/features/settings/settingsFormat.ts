import { pluralize } from '@/lib/format';

import {
  DEFAULT_RETENTION_DAYS,
  MAX_RETENTION_DAYS,
  MIN_RETENTION_DAYS,
} from '@/ipc/generated/retention';

const DAYS_PER_YEAR = 365;

/** The shortest, the default, twice the default and the longest period the app keeps. */
export const RETENTION_OPTIONS_DAYS: readonly number[] = [
  ...new Set([
    MIN_RETENTION_DAYS,
    DEFAULT_RETENTION_DAYS,
    Math.min(DEFAULT_RETENTION_DAYS * 2, MAX_RETENTION_DAYS),
    MAX_RETENTION_DAYS,
  ]),
].sort((a, b) => a - b);

export function retentionLabel(days: number): string {
  return days === DAYS_PER_YEAR ? '1 year' : pluralize(days, 'day');
}

export function fileNameOf(path: string): string {
  return path.split('/').at(-1) ?? path;
}
