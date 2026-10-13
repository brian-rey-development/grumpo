export const queryKeys = {
  reports: ['reports'] as const,
  dayReport: (date: string) => ['reports', 'day', date] as const,
  weekReport: (endDate: string) => ['reports', 'week', endDate] as const,
  limitUsage: ['limit-usage'] as const,
  focusStatus: ['focus-status'] as const,
  knownApps: ['known-apps'] as const,
  appIcon: (bundleId: string) => ['app-icon', bundleId] as const,
  settings: ['settings'] as const,
  clockStyle: ['clock-style'] as const,
  permissions: ['permissions'] as const,
  onboarding: ['onboarding'] as const,
};
