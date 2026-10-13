import { motion } from 'motion/react';

import { spring, useMotionTransition } from '@/design/motion';

import { onboardingCopy } from './copy';
import { STEPS } from './steps';

import styles from './Onboarding.module.css';

/** The current step is one pill that slides between segments, so position reads as travel. */
export function ProgressBar({ index }: { index: number }) {
  const travel = useMotionTransition(spring.snappy);

  return (
    <div
      className={styles.progress}
      role="progressbar"
      aria-label={onboardingCopy.progressLabel}
      aria-valuemin={1}
      aria-valuemax={STEPS.length}
      aria-valuenow={index + 1}
      aria-valuetext={onboardingCopy.stepOf(index + 1, STEPS.length)}
    >
      {STEPS.map((step, position) => (
        <span
          key={step}
          className={styles.segment}
          data-state={position < index ? 'done' : 'ahead'}
        >
          {position === index ? (
            <motion.span
              layoutId="onboarding-progress"
              className={styles.current}
              transition={travel}
            />
          ) : null}
        </span>
      ))}
    </div>
  );
}
