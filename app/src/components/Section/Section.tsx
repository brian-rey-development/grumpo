import { type ReactNode, useId } from 'react';

import { cx } from '@/lib/classNames';

import styles from './Section.module.css';

interface SectionProps {
  title?: string;
  description?: string;
  action?: ReactNode;
  /** `group` wraps free-form content, `list` wraps `SectionRow` items in a real list. */
  as?: 'group' | 'list';
  /** A raised panel for content that acts as one unit, such as a chart. Lists and forms stay unboxed. */
  boxed?: boolean;
  children: ReactNode;
}

export function Section({
  title,
  description,
  action,
  as = 'group',
  boxed = false,
  children,
}: SectionProps) {
  const titleId = useId();
  const hasHeader = title !== undefined || action !== undefined || description !== undefined;
  const labelledBy = title === undefined ? undefined : titleId;

  return (
    <section className={styles.section}>
      {hasHeader ? (
        <header className={styles.header}>
          <div className={styles.heading}>
            {title === undefined ? null : (
              <h2 id={titleId} className={styles.title}>
                {title}
              </h2>
            )}
            {description === undefined ? null : <p className={styles.description}>{description}</p>}
          </div>
          {action === undefined ? null : <div className={styles.actions}>{action}</div>}
        </header>
      ) : null}
      {as === 'list' ? (
        <ul className={styles.group} aria-labelledby={labelledBy}>
          {children}
        </ul>
      ) : (
        <div
          className={cx(styles.group, boxed && styles.content)}
          role={labelledBy === undefined ? undefined : 'group'}
          aria-labelledby={labelledBy}
        >
          {children}
        </div>
      )}
    </section>
  );
}
