import type { UsageListsCopy } from '@/features/usage/copy';
import { formatDuration, pluralize } from '@/lib/format';

export const todayCopy = {
  heroToday: 'Screen time today',
  heroPast: 'Screen time',
  stepper: {
    group: 'Choose a day',
    previous: 'Previous day',
    next: 'Next day',
    reset: 'Today',
  },
  loading: 'Loading your day',
  day: {
    title: 'Your day',
    chartLabel: 'Screen time by hour',
    focusNote: 'Shaded hours are when focus time was scheduled.',
    focusSummary: (ranges: string) => `Focus time was scheduled ${ranges}.`,
  },
  compare: {
    less: 'less than yesterday at this time',
    more: 'more than yesterday at this time',
    same: 'Same as yesterday at this time',
  },
  lists: {
    appsTitle: 'Apps',
    appsLabel: 'Apps used this day',
    sitesTitle: 'Sites',
    sitesLabel: 'Sites visited this day',
  } satisfies UsageListsCopy,
  empty: {
    titleToday: 'Nothing tracked yet',
    descriptionToday: "Grumpo's watching. Tracking starts as you use your Mac.",
    titlePast: 'Nothing tracked on this day',
    descriptionPast: "Grumpo didn't record any activity on this day.",
  },
  error: {
    title: "Couldn't load your day",
  },
  limits: {
    remaining: (left: number, limit: number) =>
      `${formatDuration(left)} left of ${formatDuration(limit)}`,
    reached: 'Limit reached',
    spoken: (used: number, limit: number) => `${formatDuration(used)} of ${formatDuration(limit)}`,
  },
  heldSites: {
    body: "Your saved rules couldn't be read, so Grumpo kept blocking the sites that were blocked. Add them to your rules to keep them, or unlock and save your rules to let them go.",
    action: 'Open rules',
  },
  banner: {
    label: 'Focus status',
    onDuty: 'On duty',
    onDutyUntil: (clock: string) => `On duty until ${clock}`,
    guarding: (sites: number, apps: number) => {
      if (sites === 0 && apps === 0) return 'Nothing is on the block list yet.';
      const parts = [
        ...(sites > 0 ? [pluralize(sites, 'site')] : []),
        ...(apps > 0 ? [pluralize(apps, 'app')] : []),
      ];
      return `Guarding ${parts.join(' and ')}.`;
    },
    offDuty: 'Off duty',
    nextShift: (when: string) => `Next shift starts ${when}.`,
    nextShiftToday: (clock: string) => clock,
    nextShiftTomorrow: (clock: string) => `tomorrow at ${clock}`,
    nextShiftLater: (day: string, clock: string) => `${day} at ${clock}`,
    noSchedule: 'No focus times are scheduled.',
    limitOne: (name: string) => `${name} is done for today.`,
    limitMany: (name: string, others: number) =>
      `${name} and ${others.toString()} more are done for today.`,
    limitCount: (reached: number, total: number) =>
      total === 1
        ? 'Your daily limit is used up.'
        : `${reached.toString()} of ${total.toString()} daily limits used up.`,
    unlockedUntil: (clock: string) => `Unlocked until ${clock}`,
    unlockedLeft: (left: string) => `${left} left. Grumpo is back on duty after that.`,
    unlockIn: (countdown: string) => `Unlock in ${countdown}.`,
    unlockReady: 'Unlock is ready.',
    unlockReadyBy: (clock: string) => `Unlock is ready, confirm by ${clock}.`,
    unavailableLine: "Grumpo can't block anything until his service is running.",
    actions: {
      editRules: 'Edit rules',
      editSchedule: 'Edit schedule',
      unlock: 'Unlock',
      relock: 'Relock',
    },
  },
} as const;
