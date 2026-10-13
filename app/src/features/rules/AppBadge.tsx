import { AppIcon } from '@/components/AppIcon/AppIcon';
import { SiteBadge } from '@/components/SiteBadge/SiteBadge';
import type { AppRef } from '@/ipc/generated/AppRef';
import type { LimitTarget } from '@/ipc/generated/LimitTarget';
import { useAppIcon } from '@/ipc/queries';

export function AppBadge({ app }: { app: AppRef }) {
  const icon = useAppIcon(app.bundleId);
  return <AppIcon src={icon.data ?? null} />;
}

/** The icon of a site or an app, whichever the limit is about. */
export function TargetBadge({ target }: { target: LimitTarget }) {
  if (target.kind === 'site') return <SiteBadge />;
  return <AppBadge app={target.app} />;
}
