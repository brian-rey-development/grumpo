import { useCallback, useMemo, useState } from 'react';

import { addDays, parseIsoDate, toIsoDate } from '@/lib/time';

import { useTodayIso } from './useCalendar';

export interface DateNavigation {
  today: Date;
  /** The day shown, or the last day of the 7 days shown. */
  date: Date;
  isoDate: string;
  isCurrent: boolean;
  /** False once a step back would reach past the history Grumpo still keeps. */
  canGoPrevious: boolean;
  previous: () => void;
  next: () => void;
  reset: () => void;
}

function shiftIso(isoDate: string, days: number): string {
  return toIsoDate(addDays(parseIsoDate(isoDate), days));
}

interface StepRequest {
  direction: 1 | -1;
  stepDays: number;
  todayIso: string;
  /** The oldest day that still has history, or null when that is not known. */
  earliest: string | null;
}

/** The day picked after one step, where null means today, which is followed rather than pinned. */
function stepped(current: string | null, { direction, stepDays, todayIso, earliest }: StepRequest) {
  const base = current !== null && current < todayIso ? current : todayIso;
  const target = shiftIso(base, direction * stepDays);
  if (direction === -1 && earliest !== null && target < earliest) return current;
  return target < todayIso ? target : null;
}

function earliestDay(todayIso: string, retentionDays: number | null): string | null {
  return retentionDays === null ? null : shiftIso(todayIso, -retentionDays);
}

function useStepActions(request: Omit<StepRequest, 'direction'>) {
  const { stepDays, todayIso, earliest } = request;
  const [selected, setSelected] = useState<string | null>(null);
  const step = useCallback(
    (direction: 1 | -1) => {
      setSelected((current) => stepped(current, { direction, stepDays, todayIso, earliest }));
    },
    [stepDays, todayIso, earliest],
  );
  const previous = useCallback(() => {
    step(-1);
  }, [step]);
  const next = useCallback(() => {
    step(1);
  }, [step]);
  const reset = useCallback(() => {
    setSelected(null);
  }, []);
  return { selected, previous, next, reset };
}

/**
 * Steps a date by `stepDays`, never past today and never back past `retentionDays`. Staying on the
 * present is remembered as "follow today", so a screen left open overnight rolls over to the new
 * day by itself. The result only changes when a date does, not on every minute tick.
 */
export function useDateNavigation(
  stepDays: number,
  retentionDays: number | null = null,
): DateNavigation {
  const todayIso = useTodayIso();
  const earliest = earliestDay(todayIso, retentionDays);
  const { selected, previous, next, reset } = useStepActions({ stepDays, todayIso, earliest });
  const isoDate = selected !== null && selected < todayIso ? selected : todayIso;

  return useMemo(
    () => ({
      today: parseIsoDate(todayIso),
      date: parseIsoDate(isoDate),
      isoDate,
      isCurrent: isoDate === todayIso,
      canGoPrevious: earliest === null || shiftIso(isoDate, -stepDays) >= earliest,
      previous,
      next,
      reset,
    }),
    [todayIso, isoDate, earliest, stepDays, previous, next, reset],
  );
}
