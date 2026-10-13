import { motion } from 'motion/react';
import type { ReactNode } from 'react';

import { spring, useMotionTransition } from '@/design/motion';

import styles from './ExplainedRow.module.css';

interface ExplainedRowProps {
  label: ReactNode;
  badge?: ReactNode;
  detail?: ReactNode;
  trailing?: ReactNode;
  children?: ReactNode;
}

/** A settings row whose text column can grow, so reasons and hints sit under the label while the actions stay on top. */
export function ExplainedRow({ label, badge, detail, trailing, children }: ExplainedRowProps) {
  const settle = useMotionTransition(spring.snappy);

  return (
    <motion.li className={styles.row} layout="position" transition={settle}>
      <div className={styles.text}>
        <span className={styles.heading}>
          <span className={styles.label}>{label}</span>
          {badge}
        </span>
        {detail === undefined ? null : <span className={styles.detail}>{detail}</span>}
        {children}
      </div>
      {trailing === undefined ? null : <span className={styles.trailing}>{trailing}</span>}
    </motion.li>
  );
}
