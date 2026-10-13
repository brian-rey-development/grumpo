import { ExplainedRow } from '@/features/explain/ExplainedRow';
import { StatusBadge } from '@/features/explain/StatusBadge';
import { settingsCopy } from '@/features/settings/copy';
import type { PermissionStatus } from '@/ipc/generated/PermissionStatus';

const copy = settingsCopy.permissions.firefox;

/** Firefox has no Automation access of its own, so its row only mirrors Accessibility. */
export function FirefoxRow({ accessibility }: { accessibility: PermissionStatus }) {
  const covered = accessibility === 'granted';

  return (
    <ExplainedRow
      label={settingsCopy.browsers.firefox}
      badge={
        <StatusBadge
          tone={covered ? 'ok' : 'quiet'}
          label={covered ? copy.covered : copy.needsAccessibility}
        />
      }
      detail={copy.detail}
    />
  );
}
