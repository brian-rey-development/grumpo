import { createContext, type ReactNode, useContext } from 'react';
import { createPortal } from 'react-dom';
import { useIsPresent } from 'motion/react';

interface PageActionsHost {
  /** Null until the page header has mounted the element that actions render into. */
  element: HTMLElement | null;
}

export const PageActionsHostContext = createContext<PageActionsHost | null>(null);

/** Renders its children beside the page title, from anywhere inside a screen. */
export function PageActions({ children }: { children: ReactNode }) {
  const host = useContext(PageActionsHostContext);
  const present = useIsPresent();
  if (host === null) throw new Error('PageActions must be used inside the AppShell');
  if (host.element === null || !present) return null;
  return createPortal(children, host.element);
}
