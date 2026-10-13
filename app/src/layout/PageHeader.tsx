import { type ReactNode, useMemo, useState } from 'react';

import { PageActionsHostContext } from '@/layout/PageActions';

import styles from './AppShell.module.css';

interface PageHeaderProps {
  title: string;
  children: ReactNode;
}

/** Owns the slot beside the title for one screen, so a screen fading out never clobbers the next one's. */
export function PageHeader({ title, children }: PageHeaderProps) {
  const [element, setElement] = useState<HTMLElement | null>(null);
  const host = useMemo(() => ({ element }), [element]);

  return (
    <PageActionsHostContext value={host}>
      <div className={styles.page}>
        <div className={styles.header}>
          <h1 className={styles.title}>{title}</h1>
          <div ref={setElement} className={styles.pageActions} />
        </div>
        {children}
      </div>
    </PageActionsHostContext>
  );
}
