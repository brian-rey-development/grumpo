import { Button } from '@/components/Button/Button';
import { commonCopy } from '@/copy/common';

import { usageCopy } from './copy';

import styles from './BrowserAccessHint.module.css';

export function BrowserAccessHint({ onOpenSettings }: { onOpenSettings: () => void }) {
  return (
    <p className={styles.hint}>
      <span>{usageCopy.browserAccessHint}</span>
      <Button variant="plain" size="small" onClick={onOpenSettings}>
        {commonCopy.openSettings}
      </Button>
    </p>
  );
}
