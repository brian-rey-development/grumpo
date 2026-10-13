import { type RefObject, useEffect } from 'react';

const PASTE_INPUT_TYPES: ReadonlySet<string> = new Set([
  'insertFromPaste',
  'insertFromDrop',
  'insertReplacementText',
  'insertFromYank',
]);

/** Switches off the helpers that would put text in for the person, such as autocorrect. */
export const LOCKED_ATTRIBUTES = {
  spellCheck: false,
  autoCorrect: 'off',
  autoCapitalize: 'off',
  autoComplete: 'off',
} as const;

function cancelPasteLikeInput(event: InputEvent): void {
  if (PASTE_INPUT_TYPES.has(event.inputType)) event.preventDefault();
}

/** Cancels text that arrives by paste, drop or autocorrect while `enabled`, which keeps a typed sentence honest. */
export function usePasteGuard(
  inputRef: RefObject<HTMLInputElement | null>,
  enabled: boolean,
): void {
  useEffect(() => {
    const element = inputRef.current;
    if (!enabled || element === null) return undefined;
    element.addEventListener('beforeinput', cancelPasteLikeInput);
    return () => {
      element.removeEventListener('beforeinput', cancelPasteLikeInput);
    };
  }, [inputRef, enabled]);
}
