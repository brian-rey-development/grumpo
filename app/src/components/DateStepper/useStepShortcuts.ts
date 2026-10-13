import { useEffect } from 'react';

import { useIsModalOpen } from '@/components/Dialog/modalStack';

export type StepDirection = 'previous' | 'next';

type ShortcutEvent = Pick<KeyboardEvent, 'key' | 'metaKey' | 'ctrlKey' | 'altKey' | 'shiftKey'>;

function stepForShortcut(event: ShortcutEvent): StepDirection | null {
  if (!event.metaKey || event.ctrlKey || event.altKey || event.shiftKey) return null;
  if (event.key === '[') return 'previous';
  if (event.key === ']') return 'next';
  return null;
}

/** Cmd+[ and Cmd+], which stay silent while a dialog is open like the screen shortcuts do. */
export function useStepShortcuts(onStep: (direction: StepDirection) => void): void {
  const modalOpen = useIsModalOpen();

  useEffect(() => {
    if (modalOpen) return undefined;
    function onKeyDown(event: KeyboardEvent): void {
      const direction = stepForShortcut(event);
      if (direction === null) return;
      event.preventDefault();
      onStep(direction);
    }
    window.addEventListener('keydown', onKeyDown);
    return () => {
      window.removeEventListener('keydown', onKeyDown);
    };
  }, [modalOpen, onStep]);
}
