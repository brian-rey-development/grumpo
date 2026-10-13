import type { ReactNode } from 'react';

import styles from './Section.module.css';

interface SectionRowProps {
  label: ReactNode;
  detail?: ReactNode;
  leading?: ReactNode;
  children?: ReactNode;
}

export function SectionRow({ label, detail, leading, children }: SectionRowProps) {
  return (
    <li className={styles.row}>
      {leading === undefined ? null : <span className={styles.leading}>{leading}</span>}
      <span className={styles.text}>
        <span className={styles.label}>{label}</span>
        {detail === undefined ? null : <span className={styles.detail}>{detail}</span>}
      </span>
      {children === undefined ? null : <span className={styles.trailing}>{children}</span>}
    </li>
  );
}
