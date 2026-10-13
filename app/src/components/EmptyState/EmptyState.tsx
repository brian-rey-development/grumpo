import type { ReactNode } from 'react';

import styles from './EmptyState.module.css';

interface EmptyStateProps {
  /** A mascot or picture that carries the moment. Leave it out when the view already has one. */
  visual?: ReactNode;
  title: string;
  description?: string | undefined;
  action?: ReactNode;
  /** `page` stands in for a whole screen with display type, and `inline` sits inside a section. */
  variant?: 'page' | 'inline';
  role?: 'alert' | 'status';
}

export function EmptyState({
  visual,
  title,
  description,
  action,
  variant = 'inline',
  role,
}: EmptyStateProps) {
  return (
    <div className={styles.state} data-variant={variant} role={role}>
      {visual === undefined ? null : <div className={styles.visual}>{visual}</div>}
      <div className={styles.text}>
        <h2 className={styles.title}>{title}</h2>
        {description === undefined ? null : (
          <p className={styles.description} data-selectable>
            {description}
          </p>
        )}
        {action === undefined ? null : <div className={styles.action}>{action}</div>}
      </div>
    </div>
  );
}
