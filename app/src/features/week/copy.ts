import type { UsageListsCopy } from '@/features/usage/copy';

export const weekCopy = {
  currentTitle: 'Last 7 days',
  heroCurrent: 'Screen time, last 7 days',
  heroPast: 'Screen time',
  dailyAverage: 'Daily average',
  averageNote: 'The dashed line is your daily average.',
  averageSummary: (average: string) => `The dashed line marks the daily average of ${average}.`,
  stepper: {
    group: 'Choose 7 days',
    previous: 'Previous 7 days',
    next: 'Next 7 days',
    reset: 'Last 7 days',
  },
  loading: 'Loading your last 7 days',
  sections: { daily: 'By day' },
  chartLabel: 'Screen time by day',
  lists: {
    appsTitle: 'Top apps',
    appsLabel: 'Top apps in these 7 days',
    sitesTitle: 'Top sites',
    sitesLabel: 'Top sites in these 7 days',
  } satisfies UsageListsCopy,
  empty: {
    titleCurrent: 'Nothing tracked in the last 7 days',
    titlePast: 'Nothing tracked in these 7 days',
    descriptionCurrent: 'Grumpo is watching. Tracking starts as you use your Mac.',
    descriptionPast: "Grumpo didn't record any activity in these 7 days.",
  },
  error: {
    title: "Couldn't load your last 7 days",
  },
} as const;
