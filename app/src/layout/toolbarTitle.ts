import { useIsPresent } from 'motion/react';
import { createContext, useContext, useEffect, useId } from 'react';

export interface ToolbarTitleControls {
  set: (owner: string, title: string) => void;
  clear: (owner: string) => void;
}

export const ToolbarTitleContext = createContext<ToolbarTitleControls | null>(null);

function useToolbarTitleControls(): ToolbarTitleControls {
  const controls = useContext(ToolbarTitleContext);
  if (controls === null) throw new Error('useToolbarTitle must be used inside the AppShell');
  return controls;
}

/**
 * Replaces the toolbar title while the calling screen is on show, for screens whose title depends
 * on what they display, such as a past day. A screen that is fading out never touches the title.
 */
export function useToolbarTitle(title: string | null): void {
  const controls = useToolbarTitleControls();
  const present = useIsPresent();
  const owner = useId();

  useEffect(() => {
    if (title === null || !present) return undefined;
    controls.set(owner, title);
    return () => {
      controls.clear(owner);
    };
  }, [controls, owner, title, present]);
}
