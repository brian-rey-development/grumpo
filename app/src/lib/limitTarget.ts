import type { LimitTarget } from '@/ipc/generated/LimitTarget';

/** A stable identity for a limit's target, which tells a site and an app with the same text apart. */
export function targetKey(target: LimitTarget): string {
  return target.kind === 'site' ? `site:${target.domain}` : `app:${target.app.bundleId}`;
}

export function targetName(target: LimitTarget): string {
  return target.kind === 'site' ? target.domain : target.app.name;
}
