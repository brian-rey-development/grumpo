import { Button } from '@/components/Button/Button';
import { Notice } from '@/components/Notice/Notice';
import { useHeldSites } from '@/features/status/useHeldSites';
import { useNavigation } from '@/layout/navigation';

import { todayCopy } from './copy';

const copy = todayCopy.heldSites;

/** Shown while the service keeps blocking sites it read back after losing the saved rules. */
export function HeldSitesNotice() {
  const { navigate } = useNavigation();
  const held = useHeldSites();

  if (held.length === 0) return null;

  return (
    <Notice
      tone="warning"
      role="status"
      action={
        <Button
          onClick={() => {
            navigate('rules');
          }}
        >
          {copy.action}
        </Button>
      }
    >
      {copy.body}
    </Notice>
  );
}
