import { Check } from 'lucide-react';
import { AnimatePresence, motion } from 'motion/react';

import { spring, tween, useMotionTransition } from '@/design/motion';

import styles from './StatusBadge.module.css';

const ICON_SIZE = 13;
const FROM_HIDDEN = { opacity: 0 } as const;
const POP_FROM = { scale: 0 } as const;
const POP_TO = { scale: 1 } as const;

interface StatusBadgeProps {
  tone: 'ok' | 'problem' | 'quiet';
  label: string;
}

/** Swaps with a short crossfade when the state changes, and a granted check pops in. */
export function StatusBadge({ tone, label }: StatusBadgeProps) {
  const fade = useMotionTransition(tween.fast);
  const pop = useMotionTransition(spring.squash);

  return (
    <AnimatePresence mode="wait" initial={false}>
      <motion.span
        key={`${tone}-${label}`}
        className={styles.badge}
        data-tone={tone}
        initial={FROM_HIDDEN}
        animate={{ opacity: 1 }}
        exit={FROM_HIDDEN}
        transition={fade}
      >
        {tone === 'ok' ? (
          <motion.span
            className={styles.check}
            initial={POP_FROM}
            animate={POP_TO}
            transition={pop}
          >
            <Check size={ICON_SIZE} aria-hidden="true" />
          </motion.span>
        ) : null}
        {label}
      </motion.span>
    </AnimatePresence>
  );
}
