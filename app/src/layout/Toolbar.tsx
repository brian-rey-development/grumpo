import { createContext, type ReactNode, useContext } from 'react';
import { createPortal } from 'react-dom';
import { useIsPresent } from 'motion/react';

import styles from './Toolbar.module.css';

interface ToolbarActionsHost {
  /** Null until the toolbar has mounted the element that actions render into. */
  element: HTMLElement | null;
}

export const ToolbarActionsHostContext = createContext<ToolbarActionsHost | null>(null);

interface ToolbarProps {
  actionsHostRef: (element: HTMLElement | null) => void;
}

export function Toolbar({ actionsHostRef }: ToolbarProps) {
  return (
    <header className={styles.toolbar} data-tauri-drag-region>
      <div className={styles.inner}>
        <div ref={actionsHostRef} className={styles.actions} />
      </div>
    </header>
  );
}

/** Renders its children in the trailing end of the toolbar from anywhere inside a screen. */
export function ToolbarActions({ children }: { children: ReactNode }) {
  const host = useContext(ToolbarActionsHostContext);
  const present = useIsPresent();
  if (host === null) throw new Error('ToolbarActions must be used inside the AppShell');
  if (host.element === null || !present) return null;
  return createPortal(children, host.element);
}
