import { FULL_PERCENT, toPercent } from '@/lib/percent';

import styles from './ColumnChart.module.css';

/** A span of columns, counted in column widths from the left edge. */
export interface ChartBand {
  start: number;
  end: number;
}

export interface ChartReference {
  value: number;
}

interface BandsProps {
  bands: readonly ChartBand[];
  count: number;
}

export function ChartBands({ bands, count }: BandsProps) {
  return (
    <div className={styles.bands} aria-hidden="true">
      {bands.map((band) => (
        <span
          key={`${band.start.toString()}-${band.end.toString()}`}
          className={styles.band}
          style={{
            left: `${toPercent(band.start, count).toString()}%`,
            width: `${toPercent(band.end - band.start, count).toString()}%`,
          }}
        />
      ))}
    </div>
  );
}

interface ReferenceLineProps {
  reference: ChartReference;
  max: number;
}

export function ReferenceLine({ reference, max }: ReferenceLineProps) {
  if (max <= 0) return null;
  const height = Math.min(toPercent(reference.value, max), FULL_PERCENT);
  return (
    <div
      className={styles.reference}
      style={{ bottom: `${height.toString()}%` }}
      aria-hidden="true"
    />
  );
}
