import { type CSSProperties, useId } from 'react';
import { useReducedMotion } from 'motion/react';

import { useChartNavigation } from '@/components/ChartNavigation/useChartNavigation';
import { ChartTooltip } from '@/components/ChartTooltip/ChartTooltip';
import { type ChartTick, ChartAxis } from '@/components/ColumnChart/ChartAxis';
import {
  type ChartBand,
  type ChartReference,
  ChartBands,
  ReferenceLine,
} from '@/components/ColumnChart/ChartBands';
import { type ChartHighlight, Column } from '@/components/ColumnChart/Column';
import { barScale, columnCenterPercent, sanitizeValues } from '@/components/ColumnChart/columns';
import { useRevealOnce } from '@/components/Reveal/Reveal';
import { chartStagger, tween } from '@/design/motion';

import styles from './ColumnChart.module.css';

export interface ColumnChartProps {
  label: string;
  datasetKey: string;
  values: readonly number[];
  labelFor: (index: number) => string;
  formatValue: (value: number) => string;
  ticks: readonly ChartTick[];
  highlightIndex?: number | undefined;
  /** Marks the highlighted column by coloring its bar or by outlining the whole column. */
  highlight?: ChartHighlight;
  emptyText: string;
  bands?: readonly ChartBand[];
  reference?: ChartReference | undefined;
  /** Says what the bands and the dashed line mean, for people who cannot see them. */
  summary?: string | undefined;
  size?: 'regular' | 'tall';
}

type ColumnsStyle = CSSProperties & Record<'--column-count', number>;

interface ColumnsProps {
  columns: readonly number[];
  max: number;
  label: string;
  highlightIndex: number | undefined;
  highlight: ChartHighlight;
  nav: ReturnType<typeof useChartNavigation>;
  optionId: (index: number) => string;
  describe: (index: number) => string;
  describedBy: string | undefined;
  datasetKey: string;
}

function growTransition(index: number, count: number, reduced: boolean) {
  if (reduced) return { duration: 0 };
  return { ...tween.grow, delay: chartStagger(index, count) };
}

function Columns({
  columns,
  max,
  label,
  highlightIndex,
  highlight,
  nav,
  optionId,
  describe,
  describedBy,
  datasetKey,
}: ColumnsProps) {
  const phase = useRevealOnce(`column-chart:${datasetKey}`, max > 0);
  const reduced = useReducedMotion() ?? false;
  const columnsStyle: ColumnsStyle = { '--column-count': columns.length };

  return (
    <div
      className={styles.columns}
      role="listbox"
      aria-label={label}
      aria-describedby={describedBy}
      aria-orientation="horizontal"
      aria-activedescendant={nav.focused === null ? undefined : optionId(nav.focused)}
      tabIndex={0}
      style={columnsStyle}
      onKeyDown={nav.onKeyDown}
      onFocus={nav.onFocus}
      onBlur={nav.onBlur}
    >
      {columns.map((value, index) => (
        <Column
          key={index}
          id={optionId(index)}
          label={describe(index)}
          scale={barScale(value, max, index === highlightIndex)}
          empty={value <= 0}
          active={nav.focused === index}
          highlight={index === highlightIndex ? highlight : undefined}
          phase={phase}
          transition={growTransition(index, columns.length, reduced)}
          onHover={(hovered) => {
            nav.setHovered(hovered ? index : null);
          }}
        />
      ))}
    </div>
  );
}

export function ColumnChart({
  label,
  datasetKey,
  values,
  labelFor,
  formatValue,
  ticks,
  highlightIndex,
  highlight = 'bar',
  emptyText,
  bands = [],
  reference,
  summary,
  size = 'regular',
}: ColumnChartProps) {
  const listboxId = useId();
  const summaryId = `${listboxId}-summary`;
  const columns = sanitizeValues(values);
  const max = Math.max(0, ...columns);
  const count = columns.length;
  const nav = useChartNavigation(count, highlightIndex ?? 0);
  const optionId = (index: number) => `${listboxId}-${index.toString()}`;
  const describe = (index: number) => `${labelFor(index)}, ${formatValue(columns[index] ?? 0)}`;

  return (
    <div className={styles.chart}>
      <div className={styles.plot} data-size={size}>
        <ChartBands bands={bands} count={count} />
        <Columns
          columns={columns}
          max={max}
          label={label}
          highlightIndex={highlightIndex}
          highlight={highlight}
          nav={nav}
          optionId={optionId}
          describe={describe}
          describedBy={summary === undefined ? undefined : summaryId}
          datasetKey={datasetKey}
        />
        {reference === undefined ? null : <ReferenceLine reference={reference} max={max} />}
        {nav.shown === null ? null : (
          <ChartTooltip anchorPercent={columnCenterPercent(nav.shown, count)}>
            {describe(nav.shown)}
          </ChartTooltip>
        )}
        {max === 0 ? <p className={styles.empty}>{emptyText}</p> : null}
      </div>
      <ChartAxis ticks={ticks} count={count} />
      {summary === undefined ? null : (
        <p id={summaryId} className="sr-only">
          {summary}
        </p>
      )}
    </div>
  );
}
