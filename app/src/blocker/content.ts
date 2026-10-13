import type { BlockScreen } from '@/ipc/generated/BlockScreen';
import type { BlockTarget } from '@/ipc/generated/BlockTarget';
import { MS_PER_SECOND, SECONDS_PER_MINUTE } from '@/lib/time';

import { blockerCopy, HEADLINES } from './copy';

const FIRST_HEADLINE = Math.floor(Math.random() * HEADLINES.length);

export type BlockTone = 'focus' | 'limit';

export function headlineFor(showId: number): string {
  const index = (FIRST_HEADLINE + Math.max(showId - 1, 0)) % HEADLINES.length;
  return HEADLINES[index] ?? HEADLINES[0];
}

export function targetName(target: BlockTarget): string {
  return target.kind === 'site' ? target.domain : target.name;
}

export function bodyFor({ target, reason }: BlockScreen): string {
  if (reason === 'limit') return blockerCopy.body.limit(targetName(target));
  return target.kind === 'site'
    ? blockerCopy.body.site(target.domain)
    : blockerCopy.body.app(target.name);
}

export function toneFor({ reason }: BlockScreen): BlockTone {
  return reason === 'limit' ? 'limit' : 'focus';
}

export function secondsLeft(until: number, now: Date): number {
  const nowSeconds = now.getTime() / MS_PER_SECOND;
  return Math.max(until - nowSeconds, 0);
}

export function roundUpToMinute(seconds: number): number {
  return Math.ceil(seconds / SECONDS_PER_MINUTE) * SECONDS_PER_MINUTE;
}
