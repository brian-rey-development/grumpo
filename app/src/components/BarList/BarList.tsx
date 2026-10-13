import { AnimatePresence, motion, useReducedMotion } from 'motion/react';
import { type ReactNode, useState } from 'react';

import { DurationText } from '@/components/DurationText/DurationText';
import { type RevealPhase, useRevealOnce } from '@/components/Reveal/Reveal';
import { listStagger, spring, tween, useMotionTransition } from '@/design/motion';
import { FULL_PERCENT, toPercent } from '@/lib/percent';

import styles from './BarList.module.css';

const MIN_VISIBLE_PERCENT = 1.5;
const HIDDEN_OFFSET = `${(-FULL_PERCENT).toString()}%`;

export type BarLevel = 'ok' | 'near' | 'reached';

/** A daily limit shown on a row, where the bar fills toward the limit instead of toward the longest row. */
export interface BarLimit {
  /** How much of the limit is used, from 0 to 1. */
  fraction: number;
  level: BarLevel;
  /** The words under the bar, such as how much time is left. */
  note: string;
  /** What a screen reader hears for the whole meter. */
  spoken: string;
}

export interface BarListItem {
  key: string;
  label: string;
  seconds: number;
  leading?: ReactNode;
  limit?: BarLimit;
}

interface BarListProps {
  label: string;
  /** Names the dataset for the once-per-session entrance, so rewording `label` never replays it. */
  revealKey?: string;
  items: readonly BarListItem[];
  emptyText: string;
}

interface BarRowProps {
  item: BarListItem;
  percent: number;
  index: number;
  phase: RevealPhase;
}

function barPercent(seconds: number, max: number): number {
  if (seconds <= 0 || max <= 0) return 0;
  return Math.max(toPercent(seconds, max), MIN_VISIBLE_PERCENT);
}

function limitPercent(limit: BarLimit): number {
  return Math.max(Math.min(limit.fraction, 1) * FULL_PERCENT, MIN_VISIBLE_PERCENT);
}

interface FillProps {
  percent: number;
  index: number;
  phase: RevealPhase;
  level?: BarLevel | undefined;
}

function Fill({ percent, index, phase, level }: FillProps) {
  const reduced = useReducedMotion();
  const base = useMotionTransition(spring.gentle);
  const [entered, setEntered] = useState(phase !== 'play');
  const transition = { ...base, delay: reduced || entered ? 0 : listStagger(index) };

  return (
    <motion.span
      className={styles.bar}
      data-level={level}
      initial={phase === 'idle' ? false : { x: HIDDEN_OFFSET }}
      animate={{ x: `${(percent - FULL_PERCENT).toString()}%` }}
      transition={transition}
      onAnimationComplete={() => {
        setEntered(true);
      }}
    />
  );
}

function LimitMeter({
  item,
  index,
  phase,
}: {
  item: BarListItem;
  index: number;
  phase: RevealPhase;
}) {
  const { limit } = item;
  if (limit === undefined) return null;
  return (
    <>
      <span
        className={styles.track}
        role="progressbar"
        aria-label={item.label}
        aria-valuemin={0}
        aria-valuemax={FULL_PERCENT}
        aria-valuenow={Math.round(Math.min(limit.fraction, 1) * FULL_PERCENT)}
        aria-valuetext={limit.spoken}
      >
        <Fill percent={limitPercent(limit)} index={index} phase={phase} level={limit.level} />
      </span>
      <span className={styles.note} data-level={limit.level}>
        {limit.note}
      </span>
    </>
  );
}

function BarRow({ item, percent, index, phase }: BarRowProps) {
  const settle = useMotionTransition(spring.snappy);
  const fade = useMotionTransition(tween.fast);

  return (
    <motion.li
      className={styles.row}
      layout="position"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1, transition: fade }}
      exit={{ opacity: 0, transition: fade }}
      transition={{ layout: settle }}
    >
      {item.leading === undefined ? null : <span className={styles.leading}>{item.leading}</span>}
      <span className={styles.body}>
        <span className={styles.top}>
          <span className={styles.label}>{item.label}</span>
          <DurationText seconds={item.seconds} className={styles.value} />
        </span>
        {item.limit === undefined ? (
          <span className={styles.plain} aria-hidden="true">
            <Fill percent={percent} index={index} phase={phase} />
          </span>
        ) : (
          <LimitMeter item={item} index={index} phase={phase} />
        )}
      </span>
    </motion.li>
  );
}

export function BarList({ label, revealKey = label, items, emptyText }: BarListProps) {
  const phase = useRevealOnce(`bar-list:${revealKey}`, items.length > 0);
  if (items.length === 0) {
    return <p className={styles.empty}>{emptyText}</p>;
  }
  const max = Math.max(...items.map((item) => item.seconds));

  return (
    <ol className={styles.list} aria-label={label}>
      <AnimatePresence initial={false}>
        {items.map((item, index) => (
          <BarRow
            key={item.key}
            item={item}
            percent={barPercent(item.seconds, max)}
            index={index}
            phase={phase}
          />
        ))}
      </AnimatePresence>
    </ol>
  );
}
