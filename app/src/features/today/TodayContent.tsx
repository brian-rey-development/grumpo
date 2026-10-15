import { useMemo } from 'react';

import { BrowserAccessHint } from '@/features/usage/BrowserAccessHint';
import { useBrowserAccessMissing } from '@/features/usage/browserAccess';
import { Hero } from '@/features/usage/Hero';
import { ScreenStack } from '@/features/usage/ScreenStack';
import { UsageLists } from '@/features/usage/UsageLists';
import { formatFullDate } from '@/lib/format';

import { todayCopy } from './copy';
import { DayChartSection } from './DaySections';
import { DutyBanner } from './DutyBanner';
import { HeldSitesNotice } from './HeldSitesNotice';
import { TodayEmpty } from './TodayEmpty';
import type { TodayView } from './todayModel';
import { useLimitBars } from './useLimitBars';
import { YesterdayLine } from './YesterdayLine';

interface TodayContentProps {
  view: TodayView;
  date: Date;
  isToday: boolean;
  stale: boolean;
  onOpenSettings: () => void;
}

export function TodayContent({ view, date, isToday, stale, onOpenSettings }: TodayContentProps) {
  const browserAccessMissing = useBrowserAccessMissing();
  const dateLine = useMemo(() => formatFullDate(date), [date]);
  const limitFor = useLimitBars(isToday);

  return (
    <ScreenStack stale={stale}>
      {isToday ? <DutyBanner /> : null}
      {isToday ? <HeldSitesNotice /> : null}
      {view.isEmpty ? (
        <TodayEmpty
          isToday={isToday}
          browserAccessMissing={browserAccessMissing}
          onOpenSettings={onOpenSettings}
        />
      ) : (
        <>
          <Hero
            label={isToday ? todayCopy.heroToday : todayCopy.heroPast}
            seconds={view.totalSeconds}
            revealKey={view.date}
          >
            {isToday ? <YesterdayLine hourly={view.hourly} /> : <span>{dateLine}</span>}
          </Hero>
          <DayChartSection view={view} />
          <UsageLists
            apps={view.apps}
            sites={view.sites}
            copy={todayCopy.lists}
            limitFor={limitFor}
          />
          {browserAccessMissing && view.sites.length === 0 ? (
            <BrowserAccessHint onOpenSettings={onOpenSettings} />
          ) : null}
        </>
      )}
    </ScreenStack>
  );
}
