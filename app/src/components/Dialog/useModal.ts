import { type RefObject, useEffect, useState } from 'react';

import { useModalRegistration } from '@/components/Dialog/modalStack';
import { useFocusTrap } from '@/components/Dialog/useFocusTrap';

/**
 * Everything a dialog needs from the modal stack. The hooks are declared in this order on
 * purpose, because React runs cleanups in declaration order. The app has to stop being inert and
 * the trap has to let go before the element that opened the dialog can take focus back.
 */
export function useModal(
  panelRef: RefObject<HTMLElement | null>,
  present: boolean,
  initialFocusRef?: RefObject<HTMLElement | null>,
  onEscape?: () => void,
): { isTop: boolean } {
  const { isTop } = useModalRegistration(present);
  useFocusTrap(panelRef, present && isTop, initialFocusRef);
  useEscape(present && isTop ? onEscape : undefined);
  useRestoreFocus(present);
  return { isTop };
}

function useRestoreFocus(present: boolean): void {
  const [opener] = useState(() => document.activeElement);

  useEffect(() => {
    if (!present) return undefined;
    return () => {
      if (opener instanceof HTMLElement) opener.focus();
    };
  }, [present, opener]);
}

/** Listens on the document so Escape still closes the dialog when focus has dropped to the body. */
function useEscape(onEscape: (() => void) | undefined): void {
  useEffect(() => {
    if (onEscape === undefined) return undefined;

    function onKeyDown(event: KeyboardEvent): void {
      if (event.key !== 'Escape' || event.defaultPrevented) return;
      event.stopPropagation();
      onEscape?.();
    }

    document.addEventListener('keydown', onKeyDown);
    return () => {
      document.removeEventListener('keydown', onKeyDown);
    };
  }, [onEscape]);
}
