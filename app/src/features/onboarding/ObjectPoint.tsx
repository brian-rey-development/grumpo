import type { ReactElement } from 'react';

import { cx } from '@/lib/classNames';

import styles from './Step.module.css';

export type ObjectName = 'block' | 'limit' | 'unlock';

const OUTLINE = 3;
const BOX = 48;

function NoEntry() {
  return (
    <>
      <circle cx="24" cy="24" r="18" className={styles.fillBlock} strokeWidth={OUTLINE} />
      <rect
        x="12"
        y="20"
        width="24"
        height="8"
        rx="2"
        className={styles.fillLight}
        strokeWidth={OUTLINE}
      />
    </>
  );
}

function Hourglass() {
  return (
    <>
      <path
        d="M14 11H34C34 19 27 21 27 24C27 27 34 29 34 37H14C14 29 21 27 21 24C21 21 14 19 14 11Z"
        className={styles.fillLight}
        strokeWidth={OUTLINE}
      />
      <path d="M17 36C18 31 22 29 24 27C26 29 30 31 31 36Z" className={styles.fillLimit} />
      <rect
        x="10"
        y="5"
        width="28"
        height="6"
        rx="3"
        className={styles.fillLimit}
        strokeWidth={OUTLINE}
      />
      <rect
        x="10"
        y="37"
        width="28"
        height="6"
        rx="3"
        className={styles.fillLimit}
        strokeWidth={OUTLINE}
      />
    </>
  );
}

function Padlock() {
  return (
    <>
      <path d="M16 22V15A8 8 0 0 1 32 15V17" className={styles.shackle} strokeWidth={OUTLINE} />
      <rect
        x="9"
        y="21"
        width="30"
        height="22"
        rx="5"
        className={styles.fillUnlock}
        strokeWidth={OUTLINE}
      />
      <circle cx="24" cy="30" r="3" className={styles.keyhole} />
      <rect x="22.5" y="31" width="3" height="6" rx="1" className={styles.keyhole} />
    </>
  );
}

const DRAWINGS: Record<ObjectName, () => ReactElement> = {
  block: NoEntry,
  limit: Hourglass,
  unlock: Padlock,
};

interface ObjectPointProps {
  object: ObjectName;
  title: string;
  body: string;
}

/** A list item led by a drawn object in a fixed box, so every text column starts at the same left edge. */
export function ObjectPoint({ object, title, body }: ObjectPointProps) {
  const Drawing = DRAWINGS[object];

  return (
    <li className={cx(styles.point, styles.objectPoint)}>
      <svg
        className={styles.object}
        viewBox={`0 0 ${BOX.toString()} ${BOX.toString()}`}
        aria-hidden="true"
      >
        <Drawing />
      </svg>
      <span className={styles.pointText}>
        <span className={styles.pointTitle}>{title}</span>
        <span className={styles.pointBody}>{body}</span>
      </span>
    </li>
  );
}
