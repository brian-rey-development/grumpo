import { PresenceContext } from 'motion/react';
import type { ReactNode } from 'react';

import styles from './ScreenStack.module.css';

interface ScreenStackProps {
  /** True while the previous period is still on show, so it is dimmed instead of replaced. */
  stale: boolean;
  children: ReactNode;
}

/**
 * The shell's `AnimatePresence initial={false}` tells everything that mounts inside the first
 * screen to skip its entrance, including charts that mount later when their data arrives. Leaving
 * that presence here lets the hero, chart and bars play their entrance once their data is in.
 */
export function ScreenStack({ stale, children }: ScreenStackProps) {
  return (
    <PresenceContext value={null}>
      <div className={styles.stack} aria-busy={stale} data-stale={stale || undefined}>
        {children}
      </div>
    </PresenceContext>
  );
}
