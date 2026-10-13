import { DurationText } from '@/components/DurationText/DurationText';
import { ColumnChart } from '@/components/ColumnChart/ColumnChart';
import { EmptyState } from '@/components/EmptyState/EmptyState';
import { Mascot } from '@/components/Mascot/Mascot';
import { Section } from '@/components/Section/Section';
import { formatDuration } from '@/lib/format';
import { BrowserAccessHint } from '@/features/usage/BrowserAccessHint';
import { useBrowserAccessMissing } from '@/features/usage/browserAccess';
import { weekChartData } from '@/features/usage/chartData';
import { Hero } from '@/features/usage/Hero';
import { ScreenStack } from '@/features/usage/ScreenStack';
import { UsageLists } from '@/features/usage/UsageLists';

import { weekCopy } from './copy';
import type { WeekView } from './weekModel';

interface WeekContentProps {
  view: WeekView;
  today: Date;
  isCurrent: boolean;
  stale: boolean;
  onOpenSettings: () => void;
}

function WeekEmpty({ isCurrent }: { isCurrent: boolean }) {
  const { empty } = weekCopy;
  return (
    <EmptyState
      visual={<Mascot pose="sleep" size="empty" />}
      role="status"
      title={isCurrent ? empty.titleCurrent : empty.titlePast}
      description={isCurrent ? empty.descriptionCurrent : empty.descriptionPast}
    />
  );
}

export function WeekContent({ view, today, isCurrent, stale, onOpenSettings }: WeekContentProps) {
  const browserAccessMissing = useBrowserAccessMissing();
  const showHint = browserAccessMissing && (view.isEmpty || view.sites.length === 0);

  return (
    <ScreenStack stale={stale}>
      {view.isEmpty ? (
        <Section>
          <WeekEmpty isCurrent={isCurrent} />
        </Section>
      ) : (
        <>
          <Hero
            label={isCurrent ? weekCopy.heroCurrent : weekCopy.heroPast}
            seconds={view.totalSeconds}
            revealKey={view.days.at(-1)?.date ?? ''}
          >
            {view.averageSeconds > 0 ? (
              <span>
                {weekCopy.dailyAverage} <DurationText seconds={view.averageSeconds} />
              </span>
            ) : null}
          </Hero>
          <Section boxed title={weekCopy.sections.daily} description={weekCopy.averageNote}>
            <ColumnChart
              label={weekCopy.chartLabel}
              datasetKey="week-days"
              size="tall"
              highlight="outline"
              reference={{ value: view.averageSeconds }}
              summary={weekCopy.averageSummary(formatDuration(view.averageSeconds))}
              {...weekChartData(view.days, today)}
            />
          </Section>
          <UsageLists apps={view.apps} sites={view.sites} copy={weekCopy.lists} />
        </>
      )}
      {showHint ? <BrowserAccessHint onOpenSettings={onOpenSettings} /> : null}
    </ScreenStack>
  );
}
