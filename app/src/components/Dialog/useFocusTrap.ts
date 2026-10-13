import { type RefObject, useEffect } from 'react';

const FOCUSABLE_SELECTOR = [
  'a[href]',
  'button:not([disabled])',
  'input:not([disabled])',
  'select:not([disabled])',
  'textarea:not([disabled])',
  '[tabindex]:not([tabindex="-1"])',
].join(',');

function focusableWithin(container: HTMLElement): HTMLElement[] {
  return [...container.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR)];
}

function wrapTab(panel: HTMLElement, event: KeyboardEvent): void {
  const items = focusableWithin(panel);
  const head = items[0];
  const tail = items.at(-1);
  if (head === undefined || tail === undefined) {
    event.preventDefault();
    panel.focus();
  } else if (event.shiftKey && document.activeElement === head) {
    event.preventDefault();
    tail.focus();
  } else if (!event.shiftKey && document.activeElement === tail) {
    event.preventDefault();
    head.focus();
  }
}

/** Keeps keyboard focus inside `panelRef` while `active`. Only the top-most dialog is active. */
export function useFocusTrap(
  panelRef: RefObject<HTMLElement | null>,
  active: boolean,
  initialFocusRef?: RefObject<HTMLElement | null>,
): void {
  useEffect(() => {
    const panel = panelRef.current;
    if (!active || panel === null) return undefined;

    if (!panel.contains(document.activeElement)) {
      (initialFocusRef?.current ?? focusableWithin(panel)[0] ?? panel).focus();
    }

    function onKeyDown(event: KeyboardEvent): void {
      if (event.key === 'Tab' && panel !== null) wrapTab(panel, event);
    }

    function onFocusIn(event: FocusEvent): void {
      if (panel === null || !(event.target instanceof Node) || panel.contains(event.target)) return;
      (focusableWithin(panel)[0] ?? panel).focus();
    }

    document.addEventListener('keydown', onKeyDown);
    document.addEventListener('focusin', onFocusIn);
    return () => {
      document.removeEventListener('keydown', onKeyDown);
      document.removeEventListener('focusin', onFocusIn);
    };
  }, [panelRef, active, initialFocusRef]);
}
