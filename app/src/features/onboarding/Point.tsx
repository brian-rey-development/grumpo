import type { LucideIcon } from 'lucide-react';

import styles from './Step.module.css';

const ICON_SIZE = 20;
const ICON_STROKE = 2.25;

interface PointProps {
  icon: LucideIcon;
  title: string;
  body?: string;
}

/** One line of a plain list, with a short explanation under it when the title alone isn't enough. */
export function Point({ icon: Icon, title, body }: PointProps) {
  return (
    <li className={styles.point}>
      <span className={styles.pointIcon} aria-hidden="true">
        <Icon size={ICON_SIZE} strokeWidth={ICON_STROKE} nonScalingStroke />
      </span>
      <span className={styles.pointText}>
        <span className={styles.pointTitle}>{title}</span>
        {body === undefined ? null : <span className={styles.pointBody}>{body}</span>}
      </span>
    </li>
  );
}
