import { EmptyState } from '@/components/EmptyState/EmptyState';
import { Mascot } from '@/components/Mascot/Mascot';
import { Section } from '@/components/Section/Section';
import { BrowserAccessHint } from '@/features/usage/BrowserAccessHint';

import { todayCopy } from './copy';

interface TodayEmptyProps {
  isToday: boolean;
  browserAccessMissing: boolean;
  onOpenSettings: () => void;
}

export function TodayEmpty({ isToday, browserAccessMissing, onOpenSettings }: TodayEmptyProps) {
  const { empty } = todayCopy;
  return (
    <Section>
      <EmptyState
        visual={isToday ? <Mascot pose="zen" size="empty" /> : undefined}
        role="status"
        title={isToday ? empty.titleToday : empty.titlePast}
        description={isToday ? empty.descriptionToday : empty.descriptionPast}
      />
      {browserAccessMissing ? <BrowserAccessHint onOpenSettings={onOpenSettings} /> : null}
    </Section>
  );
}
