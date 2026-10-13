import { useState } from 'react';

import { BarList } from '@/components/BarList/BarList';
import { Button } from '@/components/Button/Button';
import { Section } from '@/components/Section/Section';
import { SiteBadge } from '@/components/SiteBadge/SiteBadge';
import { commonCopy } from '@/copy/common';
import type { UsageItem } from '@/ipc/generated/UsageItem';

import { AppUsageIcon } from './AppUsageIcon';
import { usageCopy } from './copy';
import { type LimitFor, MAX_VISIBLE_ITEMS, toBarItems, visibleItems } from './usageItems';

interface UsageBarListProps {
  title: string;
  listLabel: string;
  items: readonly UsageItem[];
  limitFor?: LimitFor | undefined;
}

function leadingFor(item: UsageItem) {
  return item.kind === 'app' ? <AppUsageIcon bundleId={item.key} /> : <SiteBadge />;
}

export function UsageBarList({ title, listLabel, items, limitFor }: UsageBarListProps) {
  const [expanded, setExpanded] = useState(false);
  const canExpand = items.length > MAX_VISIBLE_ITEMS;

  const toggle = canExpand ? (
    <Button
      variant="plain"
      size="small"
      aria-expanded={expanded}
      onClick={() => {
        setExpanded((value) => !value);
      }}
    >
      {expanded ? usageCopy.showFewer : usageCopy.showAll(items.length)}
    </Button>
  ) : undefined;

  return (
    <Section title={title} action={toggle}>
      <BarList
        label={listLabel}
        items={toBarItems(visibleItems(items, expanded), leadingFor, limitFor)}
        emptyText={commonCopy.nothingTracked}
      />
    </Section>
  );
}
