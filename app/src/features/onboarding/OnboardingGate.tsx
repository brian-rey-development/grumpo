import { motion } from 'motion/react';
import type { ReactNode } from 'react';

import { tween } from '@/design/motion';
import { useOnboardingState } from '@/ipc/queries';

import { Onboarding } from './Onboarding';

import styles from './Onboarding.module.css';

interface OnboardingGateProps {
  children: ReactNode;
}

/**
 * Shows the welcome until the backend says it was finished. If that can't be read the app opens
 * as usual, because a broken welcome must never lock the person out of their rules.
 */
export function OnboardingGate({ children }: OnboardingGateProps) {
  const state = useOnboardingState();
  if (state.isPending) return <div className={styles.pending} aria-hidden="true" />;
  return (
    <motion.div
      className={styles.gate}
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={tween.fast}
    >
      {state.data?.completed === false ? <Onboarding /> : children}
    </motion.div>
  );
}
