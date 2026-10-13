import { AnimatePresence, motion } from 'motion/react';
import { type ReactNode, useEffect, useMemo, useRef, useState } from 'react';

import { tween, useMotionTransition } from '@/design/motion';
import { layoutCopy } from '@/layout/copy';
import { SCREEN_LABELS, type Screen, useNavigation } from '@/layout/navigation';
import { PageHeader } from '@/layout/PageHeader';
import { Sidebar } from '@/layout/Sidebar';
import { Toolbar, ToolbarActionsHostContext } from '@/layout/Toolbar';
import { type ToolbarTitleControls, ToolbarTitleContext } from '@/layout/toolbarTitle';

import styles from './AppShell.module.css';

const SCREEN_OFFSET_PX = 6;

interface AppShellProps {
  sidebarFooter?: ReactNode;
  /** Screens holding unsaved work, marked in the sidebar. */
  unsavedScreens?: readonly Screen[];
  children: ReactNode;
}

function useScreenAnnouncement(title: string): string {
  const previous = useRef(title);
  const [message, setMessage] = useState('');

  useEffect(() => {
    if (previous.current === title) return;
    previous.current = title;
    setMessage(title);
  }, [title]);

  return message;
}

interface TitleOverride {
  owner: string;
  title: string;
}

function useTitleOverride(): [TitleOverride | null, ToolbarTitleControls] {
  const [override, setOverride] = useState<TitleOverride | null>(null);
  const controls = useMemo<ToolbarTitleControls>(
    () => ({
      set: (owner, title) => {
        setOverride({ owner, title });
      },
      clear: (owner) => {
        setOverride((current) => (current?.owner === owner ? null : current));
      },
    }),
    [],
  );
  return [override, controls];
}

export function AppShell({ sidebarFooter, unsavedScreens, children }: AppShellProps) {
  const { screen } = useNavigation();
  const [actionsElement, setActionsElement] = useState<HTMLElement | null>(null);
  const actionsHost = useMemo(() => ({ element: actionsElement }), [actionsElement]);
  const transition = useMotionTransition(tween.fast);
  const title = SCREEN_LABELS[screen];
  const [titleOverride, titleControls] = useTitleOverride();
  const announcement = useScreenAnnouncement(title);

  return (
    <div className={styles.shell}>
      <Sidebar footer={sidebarFooter} {...(unsavedScreens && { unsaved: unsavedScreens })} />
      <main className={styles.content}>
        <Toolbar actionsHostRef={setActionsElement} />
        <div className={styles.stage}>
          <ToolbarActionsHostContext value={actionsHost}>
            <ToolbarTitleContext value={titleControls}>
              <AnimatePresence initial={false}>
                <motion.div
                  key={screen}
                  className={styles.screen}
                  data-screen={screen}
                  initial={{ opacity: 0, y: SCREEN_OFFSET_PX }}
                  animate={{ opacity: 1, y: 0, transition }}
                  exit={{ opacity: 0, transition }}
                >
                  <PageHeader title={titleOverride?.title ?? title}>{children}</PageHeader>
                </motion.div>
              </AnimatePresence>
            </ToolbarTitleContext>
          </ToolbarActionsHostContext>
        </div>
      </main>
      <div className="sr-only" role="status" aria-label={layoutCopy.currentScreen}>
        {announcement}
      </div>
    </div>
  );
}
