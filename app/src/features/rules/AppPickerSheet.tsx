import { Button } from '@/components/Button/Button';
import { Dialog } from '@/components/Dialog/Dialog';
import { LoadFailure } from '@/components/LoadFailure/LoadFailure';
import { Skeleton } from '@/components/Skeleton/Skeleton';
import { commonCopy } from '@/copy/common';
import { describeFailure } from '@/copy/failures';
import { AppListbox } from '@/features/rules/AppListbox';
import { rulesCopy } from '@/features/rules/copy';
import type { AppRef } from '@/ipc/generated/AppRef';
import { useKnownApps } from '@/ipc/queries';

import styles from './AppListbox.module.css';

const copy = rulesCopy.apps.picker;
const SKELETON_ROWS = 4;

interface AppChoiceProps {
  excluded: ReadonlySet<string>;
  onPick: (app: AppRef) => void;
}

/** Lists the apps on this Mac once they are known, with its own loading and error states. */
export function AppChoice({ excluded, onPick }: AppChoiceProps) {
  const query = useKnownApps();

  if (query.isPending) {
    return (
      <div className={styles.box} role="status" aria-label={copy.listLabel}>
        {Array.from({ length: SKELETON_ROWS }, (_, row) => (
          <Skeleton key={row} height="var(--size-icon-regular)" />
        ))}
      </div>
    );
  }
  if (query.isError) {
    return (
      <div className={styles.box}>
        <LoadFailure
          variant="inline"
          title={copy.loadFailed}
          description={describeFailure(query.error, commonCopy.tryAgainSoon)}
          retryLabel={commonCopy.retry}
          onRetry={() => {
            void query.refetch();
          }}
        />
      </div>
    );
  }
  return <AppListbox apps={query.data} excluded={excluded} onPick={onPick} />;
}

interface AppPickerSheetProps {
  open: boolean;
  onClose: () => void;
  onPick: (app: AppRef) => void;
  excluded: ReadonlySet<string>;
}

export function AppPickerSheet({ open, onClose, onPick, excluded }: AppPickerSheetProps) {
  return (
    <Dialog
      open={open}
      title={copy.title}
      onClose={onClose}
      footer={<Button onClick={onClose}>{commonCopy.cancel}</Button>}
    >
      <AppChoice
        excluded={excluded}
        onPick={(app) => {
          onPick(app);
          onClose();
        }}
      />
    </Dialog>
  );
}
