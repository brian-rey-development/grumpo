import { motion, type Transition } from 'motion/react';

import type { RevealPhase } from '@/components/Reveal/Reveal';
import { FULL_PERCENT } from '@/lib/percent';

import styles from './ColumnChart.module.css';

export type ChartHighlight = 'bar' | 'outline';

interface ColumnProps {
  id: string;
  label: string;
  scale: number;
  empty: boolean;
  active: boolean;
  highlight: ChartHighlight | undefined;
  phase: RevealPhase;
  transition: Transition;
  onHover: (hovered: boolean) => void;
}

export function Column({
  id,
  label,
  scale,
  empty,
  active,
  highlight,
  phase,
  transition,
  onHover,
}: ColumnProps) {
  return (
    <div
      id={id}
      className={styles.column}
      role="option"
      aria-selected={active}
      aria-label={label}
      data-highlight={highlight}
      data-focused={active || undefined}
      onPointerEnter={() => {
        onHover(true);
      }}
      onPointerLeave={() => {
        onHover(false);
      }}
    >
      <motion.span
        className={styles.bar}
        data-empty={empty || undefined}
        initial={phase === 'play' ? { y: `${FULL_PERCENT.toString()}%` } : false}
        animate={{ y: `${((1 - scale) * FULL_PERCENT).toString()}%` }}
        transition={transition}
      />
    </div>
  );
}
