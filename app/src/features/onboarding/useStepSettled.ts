import type { Permissions } from '@/ipc/generated/Permissions';
import type { PermissionStatus } from '@/ipc/generated/PermissionStatus';
import { useFocusStatus, usePermissions } from '@/ipc/queries';

import type { StepId } from './steps';

function accessibilityDone(status: PermissionStatus): boolean {
  return status === 'granted' || status === 'unsupported';
}

function browserAnswered(status: PermissionStatus): boolean {
  return status !== 'notDetermined' && status !== 'notRunning';
}

function permissionsSettled(permissions: Permissions): boolean {
  const asked = permissions.browsers.filter(
    (entry) => entry.installed && entry.permission !== 'unsupported',
  );
  return (
    accessibilityDone(permissions.accessibility) &&
    asked.every((entry) => browserAnswered(entry.permission))
  );
}

/** Whether a step has nothing left to do, which turns its skip into a plain Continue. */
export function useStepSettled(step: StepId): boolean {
  const status = useFocusStatus().data;
  const permissions = usePermissions().data;
  if (step === 'service') return status?.kind === 'connected';
  if (step === 'permissions') return permissions !== undefined && permissionsSettled(permissions);
  return true;
}
