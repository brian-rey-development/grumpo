import type { AppRef } from '@/ipc/generated/AppRef';
import {
  MAX_APPS,
  MAX_LIMIT_MINUTES,
  MAX_LIMITS,
  MAX_SITES,
  MAX_WINDOWS,
  MIN_LIMIT_MINUTES,
} from '@/ipc/generated/constants';
import type { Day } from '@/ipc/generated/Day';
import type { Limit } from '@/ipc/generated/Limit';
import type { LimitTarget } from '@/ipc/generated/LimitTarget';
import type { Rules } from '@/ipc/generated/Rules';
import type { Window } from '@/ipc/generated/Window';
import { targetKey } from '@/lib/limitTarget';
import { MINUTES_PER_HOUR } from '@/lib/time';

export const RULE_CAPS = {
  sites: MAX_SITES,
  apps: MAX_APPS,
  windows: MAX_WINDOWS,
  limits: MAX_LIMITS,
} as const;
const LIMIT_STEP_MINUTES = 5;
export const LIMIT_RANGE = {
  min: MIN_LIMIT_MINUTES,
  max: MAX_LIMIT_MINUTES,
  step: LIMIT_STEP_MINUTES,
} as const;
export const DEFAULT_LIMIT_MINUTES = 30;

const WEEKDAYS: Day[] = ['mon', 'tue', 'wed', 'thu', 'fri'];
const NINE_AM = 9 * MINUTES_PER_HOUR;
const FIVE_PM = 17 * MINUTES_PER_HOUR;

export function defaultWindow(): Window {
  return { days: [...WEEKDAYS], start: NINE_AM, end: FIVE_PM };
}

export function addSite(rules: Rules, domain: string): Rules {
  if (rules.blockedSites.includes(domain) || rules.blockedSites.length >= RULE_CAPS.sites) {
    return rules;
  }
  return { ...rules, blockedSites: [...rules.blockedSites, domain].sort() };
}

export function removeSite(rules: Rules, domain: string): Rules {
  return { ...rules, blockedSites: rules.blockedSites.filter((site) => site !== domain) };
}

export function addApp(rules: Rules, app: AppRef): Rules {
  const present = rules.blockedApps.some((entry) => entry.bundleId === app.bundleId);
  if (present || rules.blockedApps.length >= RULE_CAPS.apps) return rules;
  const apps = [...rules.blockedApps, app].sort((a, b) => a.name.localeCompare(b.name));
  return { ...rules, blockedApps: apps };
}

export function removeApp(rules: Rules, bundleId: string): Rules {
  return { ...rules, blockedApps: rules.blockedApps.filter((app) => app.bundleId !== bundleId) };
}

export function addWindow(rules: Rules): Rules {
  if (rules.schedule.windows.length >= RULE_CAPS.windows) return rules;
  return { ...rules, schedule: { windows: [...rules.schedule.windows, defaultWindow()] } };
}

export function updateWindow(rules: Rules, index: number, change: Partial<Window>): Rules {
  const windows = rules.schedule.windows.map((window, position) =>
    position === index ? { ...window, ...change } : window,
  );
  return { ...rules, schedule: { windows } };
}

export function removeWindow(rules: Rules, index: number): Rules {
  const windows = rules.schedule.windows.filter((_, position) => position !== index);
  return { ...rules, schedule: { windows } };
}

export function hasLimitFor(rules: Rules, target: LimitTarget): boolean {
  return rules.limits.some((limit) => targetKey(limit.target) === targetKey(target));
}

export function addLimit(rules: Rules, target: LimitTarget, dailyMinutes: number): Rules {
  if (hasLimitFor(rules, target) || rules.limits.length >= RULE_CAPS.limits) return rules;
  return { ...rules, limits: [...rules.limits, { target, dailyMinutes }] };
}

function mapLimit(rules: Rules, target: LimitTarget, change: (limit: Limit) => Limit): Rules {
  const key = targetKey(target);
  const limits = rules.limits.map((limit) =>
    targetKey(limit.target) === key ? change(limit) : limit,
  );
  return { ...rules, limits };
}

export function setLimitMinutes(rules: Rules, target: LimitTarget, dailyMinutes: number): Rules {
  return mapLimit(rules, target, (limit) => ({ ...limit, dailyMinutes }));
}

export function removeLimit(rules: Rules, target: LimitTarget): Rules {
  const key = targetKey(target);
  return { ...rules, limits: rules.limits.filter((limit) => targetKey(limit.target) !== key) };
}
