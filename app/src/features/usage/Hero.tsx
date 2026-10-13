import { motion } from 'motion/react';
import type { ReactNode } from 'react';

import { DurationText } from '@/components/DurationText/DurationText';
import { useRevealOnce } from '@/components/Reveal/Reveal';
import { tween, useMotionTransition } from '@/design/motion';

import styles from './Hero.module.css';

const RISE_PX = 8;
const HIDDEN = { opacity: 0, y: RISE_PX } as const;
const SHOWN = { opacity: 1, y: 0 } as const;

interface HeroProps {
  label: string;
  seconds: number;
  /** Names what the number measures, so it enters once for each day or week and not on a refetch. */
  revealKey: string;
  /** The quiet line under the number. It keeps its height while empty so nothing jumps. */
  children?: ReactNode;
}

function HeroBody({ label, seconds, revealKey, children }: HeroProps) {
  const phase = useRevealOnce(`hero:${revealKey}`, true);
  const transition = useMotionTransition(tween.base);

  return (
    <motion.div
      className={styles.hero}
      initial={phase === 'play' ? HIDDEN : false}
      animate={{ ...SHOWN, transition }}
    >
      <p className={styles.label}>{label}</p>
      <DurationText seconds={seconds} className={styles.value} />
      <div className={styles.line}>{children}</div>
    </motion.div>
  );
}

/** Starts over for every day or week, since each one earns its own entrance. */
export function Hero(props: HeroProps) {
  return <HeroBody key={props.revealKey} {...props} />;
}
