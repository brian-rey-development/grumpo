import { type KeyboardEvent, useState } from 'react';

const NAVIGATION_KEYS: readonly string[] = ['ArrowLeft', 'ArrowRight', 'Home', 'End'];

function nextIndex(current: number | null, key: string, count: number): number | null {
  if (count === 0) return null;
  if (key === 'Home') return 0;
  if (key === 'End') return count - 1;
  if (key === 'ArrowRight') return current === null ? 0 : Math.min(current + 1, count - 1);
  if (key === 'ArrowLeft') return current === null ? count - 1 : Math.max(current - 1, 0);
  return current;
}

interface ChartNavigation {
  /** The option the keyboard is on, or null while it is not on any. */
  focused: number | null;
  hovered: number | null;
  /** The option to describe, which is the hovered one before the focused one. */
  shown: number | null;
  setHovered: (index: number | null) => void;
  onKeyDown: (event: KeyboardEvent) => void;
  onFocus: () => void;
  onBlur: () => void;
}

/**
 * One keyboard model for every chart, where the chart is the single tab stop and the arrow keys,
 * Home and End move through its options with `aria-activedescendant`. Escape lets go of the option.
 * `startIndex` is the option focus lands on when the chart gains focus.
 */
export function useChartNavigation(
  count: number,
  startIndex: number | null = null,
): ChartNavigation {
  const [hovered, setHovered] = useState<number | null>(null);
  const [keyboardIndex, setKeyboardIndex] = useState<number | null>(null);
  const focused = keyboardIndex !== null && keyboardIndex < count ? keyboardIndex : null;

  function onKeyDown(event: KeyboardEvent): void {
    if (event.key === 'Escape' && (focused !== null || hovered !== null)) {
      event.preventDefault();
      setKeyboardIndex(null);
      setHovered(null);
      return;
    }
    if (!NAVIGATION_KEYS.includes(event.key)) return;
    event.preventDefault();
    setKeyboardIndex((index) => nextIndex(index, event.key, count));
  }

  return {
    focused,
    hovered,
    shown: hovered ?? focused,
    setHovered,
    onKeyDown,
    onFocus: () => {
      setKeyboardIndex((index) => index ?? startIndex);
    },
    onBlur: () => {
      setKeyboardIndex(null);
    },
  };
}
