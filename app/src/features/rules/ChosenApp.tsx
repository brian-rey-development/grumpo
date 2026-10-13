import { Button } from '@/components/Button/Button';
import { AppBadge } from '@/features/rules/AppBadge';
import { AppChoice } from '@/features/rules/AppPickerSheet';
import { rulesCopy } from '@/features/rules/copy';
import type { AppRef } from '@/ipc/generated/AppRef';

import styles from './AddLimitSheet.module.css';

interface ChosenAppProps {
  app: AppRef | null;
  /** Bundle ids that cannot be picked, such as apps that already have a limit. */
  excluded: ReadonlySet<string>;
  error: string | undefined;
  onPick: (app: AppRef) => void;
  onClear: () => void;
}

export function ChosenApp({ app, excluded, error, onPick, onClear }: ChosenAppProps) {
  return (
    <>
      {app === null ? (
        <AppChoice excluded={excluded} onPick={onPick} />
      ) : (
        <div className={styles.chosen}>
          <AppBadge app={app} />
          <span className={styles.chosenName}>{app.name}</span>
          <Button size="small" onClick={onClear}>
            {rulesCopy.limits.sheet.change}
          </Button>
        </div>
      )}
      {error === undefined ? null : (
        <p className={styles.error} role="alert">
          {error}
        </p>
      )}
    </>
  );
}
