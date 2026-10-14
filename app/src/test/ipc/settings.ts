import type { Permissions } from '@/ipc/generated/Permissions';
import type { Settings } from '@/ipc/generated/Settings';

export const DEFAULT_SETTINGS: Settings = {
  retentionDays: 90,
  launchAtLogin: false,
  autoUpdate: true,
};

export const DEFAULT_PERMISSIONS: Permissions = {
  accessibility: 'granted',
  browsers: [
    { browser: 'chrome', installed: true, permission: 'notDetermined' },
    { browser: 'safari', installed: true, permission: 'denied' },
    { browser: 'brave', installed: false, permission: 'notDetermined' },
  ],
};

export const ALLOWED_PERMISSIONS: Permissions = {
  accessibility: 'granted',
  browsers: [
    { browser: 'chrome', installed: true, permission: 'granted' },
    { browser: 'firefox', installed: true, permission: 'unsupported' },
    { browser: 'brave', installed: false, permission: 'notDetermined' },
  ],
};
