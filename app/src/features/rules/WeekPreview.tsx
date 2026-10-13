import { coverageByDay } from '@/features/rules/coverage';
import { summarizeWeek } from '@/features/rules/weekSummary';
import type { Schedule } from '@/ipc/generated/Schedule';
import { useClockOptions } from '@/ipc/queries';
import { formatClockFromSeconds } from '@/lib/format';
import { toPercent } from '@/lib/percent';
import { MINUTES_PER_DAY, MINUTES_PER_HOUR, SECONDS_PER_HOUR } from '@/lib/time';
import { dayName, WEEK_DAYS } from '@/lib/weekDays';

import styles from './WeekPreview.module.css';

const TICK_HOURS = [0, 6, 12, 18] as const;

function percent(minutes: number): string {
  return `${toPercent(minutes, MINUTES_PER_DAY).toString()}%`;
}

/** Seven thin tracks, one per day, with an orange band wherever Grumpo is on duty. */
export function WeekPreview({ schedule }: { schedule: Schedule }) {
  const week = coverageByDay(schedule);
  const clock = useClockOptions();

  return (
    <div className={styles.grid} role="img" aria-label={summarizeWeek(schedule, clock)}>
      <div className={styles.axis} aria-hidden="true">
        {TICK_HOURS.map((hour) => (
          <span
            key={hour}
            className={styles.tick}
            style={{ left: percent(hour * MINUTES_PER_HOUR) }}
          >
            {formatClockFromSeconds(hour * SECONDS_PER_HOUR, clock)}
          </span>
        ))}
      </div>
      {WEEK_DAYS.map((day) => (
        <div key={day} className={styles.row} aria-hidden="true">
          <span className={styles.day}>{dayName(day, 'short')}</span>
          <span className={styles.track}>
            {week[day].map(({ start, end }) => (
              <span
                key={start}
                className={styles.span}
                style={{ left: percent(start), width: percent(end - start) }}
              />
            ))}
          </span>
        </div>
      ))}
    </div>
  );
}
