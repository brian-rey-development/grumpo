import type { ReactNode } from 'react';

import { FULL_PERCENT } from '@/lib/percent';

import styles from './ChartTooltip.module.css';

const EDGE_PERCENT = 12;
const CENTER_PERCENT = 50;

interface ChartTooltipProps {
  anchorPercent?: number;
  children: ReactNode;
}

function clampTooltipAnchor(percent: number): number {
  return Math.min(Math.max(percent, EDGE_PERCENT), FULL_PERCENT - EDGE_PERCENT);
}

export function ChartTooltip({ anchorPercent = CENTER_PERCENT, children }: ChartTooltipProps) {
  return (
    <span
      className={styles.tooltip}
      style={{ left: `${clampTooltipAnchor(anchorPercent).toString()}%` }}
      aria-hidden="true"
    >
      {children}
    </span>
  );
}
