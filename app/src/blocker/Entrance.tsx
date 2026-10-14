import { motion, useReducedMotion } from 'motion/react';
import { createContext, type ReactNode, useContext, useEffect, useState } from 'react';

import { duration, tween } from '@/design/motion';

const RISE_OFFSET_PX = 12;
const LAST_RISE_ORDER = 2;
const FIRST_RISE_DELAY_SECONDS = 0.25;
const RISE_STAGGER_SECONDS = 0.05;
const MS_PER_SECOND = 1000;

function riseDelay(order: number): number {
  return FIRST_RISE_DELAY_SECONDS + order * RISE_STAGGER_SECONDS;
}

const ENTRANCE_MS = Math.ceil((riseDelay(LAST_RISE_ORDER) + duration.base) * MS_PER_SECOND);

const EntranceContext = createContext(false);

/** True while a show is still staging its lines, so a line added afterwards fades in without the stagger. */
export function EntranceProvider({ children }: { children: ReactNode }) {
  const [entering, setEntering] = useState(true);

  useEffect(() => {
    const timer = setTimeout(() => {
      setEntering(false);
    }, ENTRANCE_MS);
    return () => {
      clearTimeout(timer);
    };
  }, []);

  return <EntranceContext value={entering}>{children}</EntranceContext>;
}

interface RiseProps {
  order: number;
  className?: string | undefined;
  children: ReactNode;
}

/** A line that rises after the mascot lands, one beat after the line before it. */
export function Rise({ order, className, children }: RiseProps) {
  const reduced = useReducedMotion();
  const staged = useContext(EntranceContext) && !reduced;
  const delay = staged ? riseDelay(order) : 0;

  return (
    <motion.div
      className={className}
      initial={{ opacity: 0, y: staged ? RISE_OFFSET_PX : 0 }}
      animate={{ opacity: 1, y: 0, transition: { ...tween.base, delay } }}
      exit={{ opacity: 0, transition: tween.fast }}
    >
      {children}
    </motion.div>
  );
}
