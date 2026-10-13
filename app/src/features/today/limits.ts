import type { LimitTarget } from '@/ipc/generated/LimitTarget';
import type { LimitUsage } from '@/ipc/generated/LimitUsage';
import { targetKey, targetName } from '@/lib/limitTarget';
import { SECONDS_PER_MINUTE } from '@/lib/time';

type LimitLevel = 'ok' | 'near' | 'reached';

/** Within this much time of the limit the bar turns amber. */
const NEAR_LIMIT_SECONDS = 5 * SECONDS_PER_MINUTE;

export interface LimitProgress {
  key: string;
  name: string;
  level: LimitLevel;
  fraction: number;
  remainingSeconds: number;
  target: LimitTarget;
  usedSeconds: number;
  limitSeconds: number;
}

/** Whether the limit is reached is the service's call, since it is what blocks the target. */
function levelOf(reached: boolean, remainingSeconds: number): LimitLevel {
  if (reached) return 'reached';
  return remainingSeconds <= NEAR_LIMIT_SECONDS ? 'near' : 'ok';
}

export function describeLimit(usage: LimitUsage): LimitProgress {
  const remainingSeconds = Math.max(usage.limitSeconds - usage.usedSeconds, 0);
  const fraction = usage.limitSeconds > 0 ? Math.min(usage.usedSeconds / usage.limitSeconds, 1) : 1;
  return {
    key: targetKey(usage.target),
    name: targetName(usage.target),
    level: levelOf(usage.reached, remainingSeconds),
    fraction: Math.max(fraction, 0),
    remainingSeconds,
    target: usage.target,
    usedSeconds: usage.usedSeconds,
    limitSeconds: usage.limitSeconds,
  };
}
