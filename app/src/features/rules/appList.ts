import type { AppRef } from '@/ipc/generated/AppRef';
import type { KnownApp } from '@/ipc/generated/KnownApp';

/** Apps that match the search and are not excluded, running ones first and then by name. */
export function filterApps(
  apps: readonly KnownApp[],
  excluded: ReadonlySet<string>,
  query: string,
): KnownApp[] {
  const needle = query.trim().toLowerCase();
  return apps
    .filter(({ app }) => !excluded.has(app.bundleId))
    .filter(
      ({ app }) =>
        needle.length === 0 ||
        app.name.toLowerCase().includes(needle) ||
        app.bundleId.toLowerCase().includes(needle),
    )
    .sort((a, b) => {
      if (a.running !== b.running) return a.running ? -1 : 1;
      return a.app.name.localeCompare(b.app.name);
    });
}

export function optionId(listId: string, app: AppRef): string {
  return `${listId}-${app.bundleId}`;
}
