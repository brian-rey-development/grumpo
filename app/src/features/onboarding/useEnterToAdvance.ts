import { useEffect } from 'react';

const OWN_ENTER = 'button, a, summary, input, select, textarea, [role="button"]';

function isPlainEnter(event: KeyboardEvent): boolean {
  return event.key === 'Enter' && !event.repeat && !event.defaultPrevented && !event.isComposing;
}

function hasOwnEnter(target: EventTarget | null): boolean {
  return target instanceof Element && target.closest(OWN_ENTER) !== null;
}

/** Return moves on, unless the focused control already does something with it or work is still running. */
export function useEnterToAdvance(advance: () => void, busy: boolean): void {
  useEffect(() => {
    function onKeyDown(event: KeyboardEvent): void {
      if (busy || !isPlainEnter(event) || hasOwnEnter(event.target)) return;
      event.preventDefault();
      advance();
    }
    window.addEventListener('keydown', onKeyDown);
    return () => {
      window.removeEventListener('keydown', onKeyDown);
    };
  }, [advance, busy]);
}
