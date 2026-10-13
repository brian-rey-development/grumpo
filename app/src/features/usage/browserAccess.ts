import type { Permissions } from '@/ipc/generated/Permissions';
import { usePermissions } from '@/ipc/queries';

/** True when an installed browser has not granted the access that site tracking needs. */
function isBrowserAccessMissing(permissions: Permissions | undefined): boolean {
  return (
    permissions?.browsers.some(
      (browser) =>
        browser.installed &&
        (browser.permission === 'denied' || browser.permission === 'notDetermined'),
    ) ?? false
  );
}

export function useBrowserAccessMissing(): boolean {
  return isBrowserAccessMissing(usePermissions().data);
}
