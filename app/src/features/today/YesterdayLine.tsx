import { DurationText } from '@/components/DurationText/DurationText';
import { useDayReport } from '@/ipc/queries';
import { MS_PER_MINUTE, addDays, toIsoDate } from '@/lib/time';
import { useNow } from '@/lib/useNow';

import { compareThroughHour } from './comparison';
import { todayCopy } from './copy';

const copy = todayCopy.compare;

interface YesterdayLineProps {
  hourly: readonly number[];
}

/** How today is going against yesterday, once there is a finished hour and a yesterday to compare. */
export function YesterdayLine({ hourly }: YesterdayLineProps) {
  const now = useNow(MS_PER_MINUTE);
  const yesterday = useDayReport(toIsoDate(addDays(now, -1)));
  const result =
    yesterday.data === undefined || yesterday.isPlaceholderData
      ? null
      : compareThroughHour(hourly, yesterday.data.hourly, now.getHours());
  if (result === null) return null;
  if (result.kind === 'same') return <span>{copy.same}</span>;

  return (
    <span>
      <DurationText seconds={result.seconds} /> {result.kind === 'less' ? copy.less : copy.more}
    </span>
  );
}
