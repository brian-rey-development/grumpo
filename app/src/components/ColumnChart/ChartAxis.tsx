import { DurationText } from '@/components/DurationText/DurationText';
import { columnCenterPercent, columnStartPercent } from '@/components/ColumnChart/columns';

import styles from './ColumnChart.module.css';

export interface ChartTick {
  index: number;
  label: string;
  /** A second, quieter line under the label, shown as a duration in the shared duration style. */
  captionSeconds?: number;
  align?: 'start' | 'center';
}

interface AxisProps {
  ticks: readonly ChartTick[];
  count: number;
}

export function ChartAxis({ ticks, count }: AxisProps) {
  const hasCaptions = ticks.some((tick) => tick.captionSeconds !== undefined);
  return (
    <div className={styles.axis} data-captions={hasCaptions || undefined} aria-hidden="true">
      {ticks.map((tick) => {
        const percent =
          tick.align === 'center'
            ? columnCenterPercent(tick.index, count)
            : columnStartPercent(tick.index, count);
        return (
          <span
            key={tick.index}
            className={styles.tick}
            data-align={tick.align ?? 'start'}
            style={{ left: `${percent.toString()}%` }}
          >
            {tick.label}
            {tick.captionSeconds === undefined || tick.captionSeconds <= 0 ? null : (
              <span className={styles.caption}>
                <DurationText seconds={tick.captionSeconds} />
              </span>
            )}
          </span>
        );
      })}
    </div>
  );
}
