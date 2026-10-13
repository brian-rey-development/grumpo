import type { MascotPose } from '@/components/Mascot/Mascot';
import { commonCopy } from '@/copy/common';
import type { FocusState, UnlockWait } from '@/features/status/focusState';
import type { Rules } from '@/ipc/generated/Rules';
import {
  type ClockOptions,
  formatClock,
  formatCountdown,
  formatDuration,
  formatTimeOfDay,
} from '@/lib/format';
import { MS_PER_SECOND } from '@/lib/time';
import { dayName } from '@/lib/weekDays';

import { todayCopy } from './copy';
import { nextShiftStart } from './nextShift';

const copy = todayCopy.banner;

export type BannerTone = 'focus' | 'off' | 'limit' | 'unlocked' | 'unavailable';
export type BannerAction = 'editRules' | 'editSchedule' | 'unlock' | 'relock' | 'openSettings';

export interface BannerModel {
  tone: BannerTone;
  pose: MascotPose;
  headline: string;
  line: string;
  actions: readonly BannerAction[];
}

interface BannerInput {
  state: FocusState;
  unlockWait: UnlockWait | null;
  rules: Rules | null;
  now: Date;
  clock: ClockOptions;
}

function waitLine(wait: UnlockWait, now: Date, clock: ClockOptions): string | null {
  if (wait.kind === 'ready') return copy.unlockReadyBy(formatClock(wait.confirmBy, clock));
  const remaining = (wait.readyAt.getTime() - now.getTime()) / MS_PER_SECOND;
  return remaining > 0
    ? copy.unlockIn(formatCountdown(remaining, { padMinutes: false }))
    : copy.unlockReady;
}

function focusBanner(until: Date | null, input: BannerInput): BannerModel {
  const { rules, unlockWait, now, clock } = input;
  const waiting = unlockWait === null ? null : waitLine(unlockWait, now, clock);
  const guarding = copy.guarding(rules?.blockedSites.length ?? 0, rules?.blockedApps.length ?? 0);
  return {
    tone: 'focus',
    pose: waiting === null ? 'laptop' : 'hourglass',
    headline: until === null ? copy.onDuty : copy.onDutyUntil(formatClock(until, clock)),
    line: waiting ?? guarding,
    actions: ['editRules', 'unlock'],
  };
}

function limitBanner(targets: readonly string[], rules: Rules | null): BannerModel {
  const [first = '', ...others] = targets;
  return {
    tone: 'limit',
    pose: 'armsCrossed',
    headline: others.length > 0 ? copy.limitMany(first, others.length) : copy.limitOne(first),
    line: copy.limitCount(targets.length, Math.max(rules?.limits.length ?? 0, targets.length)),
    actions: ['editRules', 'unlock'],
  };
}

function unlockedBanner(until: Date, input: BannerInput): BannerModel {
  const left = formatDuration((until.getTime() - input.now.getTime()) / MS_PER_SECOND);
  return {
    tone: 'unlocked',
    pose: 'zen',
    headline: copy.unlockedUntil(formatClock(until, input.clock)),
    line: copy.unlockedLeft(left),
    actions: ['relock'],
  };
}

function whenText(rules: Rules, now: Date, clock: ClockOptions): string | null {
  const start = nextShiftStart(rules.schedule.windows, now);
  if (start === null) return null;
  const time = formatTimeOfDay(start.minutes, undefined, clock);
  if (start.daysAhead === 0) return copy.nextShiftToday(time);
  if (start.daysAhead === 1) return copy.nextShiftTomorrow(time);
  return copy.nextShiftLater(dayName(start.day, 'long'), time);
}

function offBanner({ rules, now, clock }: BannerInput): BannerModel {
  const when = rules === null ? null : whenText(rules, now, clock);
  return {
    tone: 'off',
    pose: 'sleep',
    headline: copy.offDuty,
    line: when === null ? copy.noSchedule : copy.nextShift(when),
    actions: ['editSchedule'],
  };
}

function unavailableBanner(): BannerModel {
  return {
    tone: 'unavailable',
    pose: 'stop',
    headline: commonCopy.serviceNotRunning,
    line: copy.unavailableLine,
    actions: ['openSettings'],
  };
}

export function describeBanner(input: BannerInput): BannerModel {
  const { state } = input;
  switch (state.kind) {
    case 'focus':
      return focusBanner(state.until, input);
    case 'limit':
      return limitBanner(state.targets, input.rules);
    case 'unlocked':
      return unlockedBanner(state.until, input);
    case 'free':
      return offBanner(input);
    case 'unavailable':
      return unavailableBanner();
  }
}
