import type { AppRef } from '@/ipc/generated/AppRef';
import type { KnownApp } from '@/ipc/generated/KnownApp';
import type { LimitTarget } from '@/ipc/generated/LimitTarget';
import type { LimitUsage } from '@/ipc/generated/LimitUsage';
import type { Rules } from '@/ipc/generated/Rules';

export const EMPTY_RULES: Rules = {
  blockedSites: [],
  blockedApps: [],
  schedule: { windows: [] },
  limits: [],
};

export const SLACK: AppRef = { bundleId: 'com.tinyspeck.slackmacgap', name: 'Slack' };
export const MAIL: AppRef = { bundleId: 'com.apple.mail', name: 'Mail' };
export const SAFARI: AppRef = { bundleId: 'com.apple.Safari', name: 'Safari' };
export const NOTES: AppRef = { bundleId: 'com.apple.Notes', name: 'Notes' };

export const YOUTUBE: LimitTarget = { kind: 'site', domain: 'youtube.com' };
export const SLACK_TARGET: LimitTarget = { kind: 'app', app: SLACK };

export const SAMPLE_RULES: Rules = {
  blockedSites: ['reddit.com', 'youtube.com'],
  blockedApps: [SLACK],
  schedule: { windows: [{ days: ['mon', 'tue', 'wed', 'thu', 'fri'], start: 540, end: 1020 }] },
  limits: [{ target: { kind: 'site', domain: 'youtube.com' }, dailyMinutes: 30 }],
};

export const KNOWN_APPS: KnownApp[] = [
  { app: NOTES, running: false },
  { app: SLACK, running: false },
  { app: MAIL, running: false },
  { app: SAFARI, running: true },
];

export const YOUTUBE_USAGE: LimitUsage[] = [
  {
    target: { kind: 'site', domain: 'youtube.com' },
    usedSeconds: 2_280,
    limitSeconds: 1_800,
    reached: true,
  },
];
