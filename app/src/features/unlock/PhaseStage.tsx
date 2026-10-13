import { AnimatePresence, motion } from 'motion/react';
import { type ReactNode, useEffect, useRef } from 'react';

import { tween, useMotionTransition } from '@/design/motion';

const FOCUSABLE = 'button:not(:disabled), input:not(:disabled)';
const PRIORITY_TARGET = '[data-autofocus]';

/** Puts focus on the new phase's main control when the one that had it just left the page. */
function FocusOnEnter({ children }: { children: ReactNode }) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const node = ref.current;
    const panel = node?.closest('[role="dialog"]');
    const active = document.activeElement;
    if (node === null || (panel?.contains(active) === true && active !== panel)) return;
    node.querySelector<HTMLElement>(PRIORITY_TARGET)?.focus();
    if (!node.contains(document.activeElement)) node.querySelector<HTMLElement>(FOCUSABLE)?.focus();
  }, []);

  return <div ref={ref}>{children}</div>;
}

interface PhaseStageProps {
  /** Changes whenever the content should swap, and stays put while the same phase updates. */
  phaseKey: string;
  children: ReactNode;
}

/**
 * Fades one phase out and the next in. The sheet takes the new content's height in one step while
 * the old phase is gone, so no layout property is ever tweened.
 */
export function PhaseStage({ phaseKey, children }: PhaseStageProps) {
  const fadeIn = useMotionTransition(tween.base);
  const fadeOut = useMotionTransition(tween.fast);

  return (
    <div>
      <AnimatePresence mode="wait" initial={false}>
        <motion.div
          key={phaseKey}
          initial={{ opacity: 0 }}
          animate={{ opacity: 1, transition: fadeIn }}
          exit={{ opacity: 0, transition: fadeOut }}
        >
          <FocusOnEnter>{children}</FocusOnEnter>
        </motion.div>
      </AnimatePresence>
    </div>
  );
}
