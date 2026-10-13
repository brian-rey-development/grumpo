import { ColumnChart } from '@/components/ColumnChart/ColumnChart';
import { Section } from '@/components/Section/Section';
import { focusBands, hourlyChartData } from '@/features/usage/chartData';
import { useClockOptions } from '@/ipc/queries';
import { formatClockRange } from '@/lib/format';

import { todayCopy } from './copy';
import type { TodayView } from './todayModel';

const copy = todayCopy.day;

export function DayChartSection({ view }: { view: TodayView }) {
  const clock = useClockOptions();
  const hasWindows = view.focusWindows.length > 0;
  const ranges = view.focusWindows.map(({ start, end }) => formatClockRange(start, end, clock));

  return (
    <Section boxed title={copy.title} {...(hasWindows && { description: copy.focusNote })}>
      <ColumnChart
        label={copy.chartLabel}
        datasetKey="day-hourly"
        bands={focusBands(view.focusWindows)}
        summary={hasWindows ? copy.focusSummary(ranges.join(', ')) : undefined}
        {...hourlyChartData(view.hourly, view.currentHour, clock)}
      />
    </Section>
  );
}
