import { useMemo } from 'react';

import type { BarLimit } from '@/components/BarList/BarList';
import type { LimitFor } from '@/features/usage/usageItems';
import { useLimitUsage } from '@/ipc/queries';

import { todayCopy } from './copy';
import { describeLimit, type LimitProgress } from './limits';

const copy = todayCopy.limits;

function toBarLimit(progress: LimitProgress): BarLimit {
  const { level, fraction, usedSeconds, limitSeconds, remainingSeconds } = progress;
  return {
    fraction,
    level,
    note: level === 'reached' ? copy.reached : copy.remaining(remainingSeconds, limitSeconds),
    spoken: copy.spoken(usedSeconds, limitSeconds),
  };
}

function keyOf(kind: 'app' | 'site', key: string): string {
  return `${kind}:${key}`;
}

/** Finds the daily limit that applies to a row of apps or sites, so it can show its progress. */
export function useLimitBars(enabled: boolean): LimitFor {
  const { data } = useLimitUsage();
  return useMemo(() => {
    const bars = new Map<string, BarLimit>();
    for (const usage of enabled ? (data ?? []) : []) {
      const progress = describeLimit(usage);
      bars.set(progress.key, toBarLimit(progress));
    }
    return (item) => bars.get(keyOf(item.kind, item.key));
  }, [data, enabled]);
}
