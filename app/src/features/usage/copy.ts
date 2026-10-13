export interface UsageListsCopy {
  appsTitle: string;
  appsLabel: string;
  sitesTitle: string;
  sitesLabel: string;
}

export const usageCopy = {
  loadFailedDescription: "Grumpo couldn't read your screen time. Try again in a moment.",
  showAll: (count: number) => `Show all ${count.toString()}`,
  showFewer: 'Show fewer',
  browserAccessHint: 'Allow browser access in Settings to see sites.',
} as const;
