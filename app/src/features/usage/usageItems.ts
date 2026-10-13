import type { ReactNode } from 'react';

import type { BarLimit, BarListItem } from '@/components/BarList/BarList';
import type { UsageItem } from '@/ipc/generated/UsageItem';

export const MAX_VISIBLE_ITEMS = 8;

export function sortByUsage(items: readonly UsageItem[]): UsageItem[] {
  return [...items].sort((a, b) => b.seconds - a.seconds);
}

export function visibleItems<T>(items: readonly T[], expanded: boolean): T[] {
  return expanded ? [...items] : items.slice(0, MAX_VISIBLE_ITEMS);
}

export type LimitFor = (item: UsageItem) => BarLimit | undefined;

export function toBarItems(
  items: readonly UsageItem[],
  leadingFor: (item: UsageItem) => ReactNode,
  limitFor?: LimitFor,
): BarListItem[] {
  return items.map((item) => {
    const limit = limitFor?.(item);
    return {
      key: item.key,
      label: item.label,
      seconds: item.seconds,
      leading: leadingFor(item),
      ...(limit && { limit }),
    };
  });
}
