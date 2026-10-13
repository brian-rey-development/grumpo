import type { Rules } from '@/ipc/generated/Rules';

export type WindowProblem = 'noDays' | 'notForward';

/** What the service would refuse in each focus time, found before asking it. */
export function windowProblems(rules: Rules): (WindowProblem | null)[] {
  return rules.schedule.windows.map((window) => {
    if (window.days.length === 0) return 'noDays';
    if (window.start >= window.end) return 'notForward';
    return null;
  });
}

export function hasProblems(rules: Rules): boolean {
  return windowProblems(rules).some((problem) => problem !== null);
}
