import { CircleAlert, Info, LockKeyhole, type LucideIcon } from 'lucide-react';
import type { ReactNode } from 'react';

import { cx } from '@/lib/classNames';

import styles from './Notice.module.css';

const ICON_SIZE = 16;

type NoticeTone = 'info' | 'warning' | 'danger';

const ICONS: Record<NoticeTone, LucideIcon> = {
  info: Info,
  warning: LockKeyhole,
  danger: CircleAlert,
};

interface NoticeProps {
  tone?: NoticeTone;
  title?: string;
  /** Warnings and errors that appear after an action should use `alert`. */
  role?: 'alert' | 'status';
  action?: ReactNode;
  children: ReactNode;
}

export function Notice({ tone = 'info', title, role, action, children }: NoticeProps) {
  const Icon = ICONS[tone];

  return (
    <div className={cx(styles.notice, styles[tone])} role={role}>
      <Icon className={styles.icon} size={ICON_SIZE} aria-hidden="true" />
      <div className={styles.body}>
        {title === undefined ? null : <p className={styles.title}>{title}</p>}
        <p className={styles.text} data-selectable>
          {children}
        </p>
      </div>
      {action === undefined ? null : <div className={styles.action}>{action}</div>}
    </div>
  );
}
