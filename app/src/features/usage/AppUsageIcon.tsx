import { AppIcon } from '@/components/AppIcon/AppIcon';
import { useAppIcon } from '@/ipc/queries';

export function AppUsageIcon({ bundleId }: { bundleId: string }) {
  const icon = useAppIcon(bundleId);
  return <AppIcon src={icon.data ?? null} />;
}
