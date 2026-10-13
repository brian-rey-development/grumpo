import type { UsageItem } from '@/ipc/generated/UsageItem';

import type { UsageListsCopy } from './copy';
import { UsageBarList } from './UsageBarList';
import { type LimitFor, sortByUsage } from './usageItems';

import styles from './UsageLists.module.css';

interface UsageListsProps {
  apps: readonly UsageItem[];
  sites: readonly UsageItem[];
  copy: UsageListsCopy;
  limitFor?: LimitFor | undefined;
}

/** Apps and sites side by side, stacking when the window is too narrow for two columns. */
export function UsageLists({ apps, sites, copy, limitFor }: UsageListsProps) {
  return (
    <div className={styles.columns}>
      <UsageBarList
        title={copy.appsTitle}
        listLabel={copy.appsLabel}
        items={sortByUsage(apps)}
        limitFor={limitFor}
      />
      <UsageBarList
        title={copy.sitesTitle}
        listLabel={copy.sitesLabel}
        items={sortByUsage(sites)}
        limitFor={limitFor}
      />
    </div>
  );
}
