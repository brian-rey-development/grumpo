import type { Rules } from '@/ipc/generated/Rules';
import { targetKey } from '@/lib/limitTarget';
import { sortDays } from '@/lib/weekDays';

function canonical(rules: Rules): string {
  return JSON.stringify({
    sites: [...rules.blockedSites].sort(),
    apps: rules.blockedApps.map((app) => app.bundleId).sort(),
    windows: rules.schedule.windows
      .map((window) => JSON.stringify([sortDays(window.days), window.start, window.end]))
      .sort(),
    limits: rules.limits
      .map((limit) => `${targetKey(limit.target)}=${limit.dailyMinutes.toString()}`)
      .sort(),
  });
}

/** Compares what the rules mean, so a different order of the same entries is not a change. */
export function rulesEqual(left: Rules, right: Rules): boolean {
  return canonical(left) === canonical(right);
}
