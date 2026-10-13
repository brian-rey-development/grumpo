import { act, renderHook } from '@testing-library/react';
import type { KeyboardEvent } from 'react';
import { expect, it, vi } from 'vitest';

import { useChartNavigation } from '@/components/ChartNavigation/useChartNavigation';

function press(key: string, preventDefault = vi.fn()): KeyboardEvent {
  return { key, preventDefault } as unknown as KeyboardEvent;
}

function setup(count: number, startIndex?: number | null) {
  return renderHook(() => useChartNavigation(count, startIndex));
}

it('starts on nothing and walks right and left within the bounds', () => {
  const { result } = setup(3);
  expect(result.current.focused).toBeNull();

  act(() => {
    result.current.onKeyDown(press('ArrowRight'));
  });
  expect(result.current.focused).toBe(0);

  act(() => {
    result.current.onKeyDown(press('ArrowRight'));
    result.current.onKeyDown(press('ArrowRight'));
  });
  act(() => {
    result.current.onKeyDown(press('ArrowRight'));
  });
  expect(result.current.focused).toBe(2);

  act(() => {
    result.current.onKeyDown(press('ArrowLeft'));
  });
  expect(result.current.focused).toBe(1);
});

it('enters from the right with the left arrow, and jumps with Home and End', () => {
  const { result } = setup(5);

  act(() => {
    result.current.onKeyDown(press('ArrowLeft'));
  });
  expect(result.current.focused).toBe(4);

  act(() => {
    result.current.onKeyDown(press('Home'));
  });
  expect(result.current.focused).toBe(0);

  act(() => {
    result.current.onKeyDown(press('End'));
  });
  expect(result.current.focused).toBe(4);
});

it('takes over the navigation keys only, and ignores them when there is nothing to walk', () => {
  const { result } = setup(0);
  const preventOther = vi.fn();
  const preventArrow = vi.fn();

  act(() => {
    result.current.onKeyDown(press('a', preventOther));
    result.current.onKeyDown(press('ArrowRight', preventArrow));
  });

  expect(preventOther).not.toHaveBeenCalled();
  expect(preventArrow).toHaveBeenCalledOnce();
  expect(result.current.focused).toBeNull();
});

it('lands on the start index when the chart gains focus, and lets go on blur', () => {
  const { result } = setup(24, 9);

  act(() => {
    result.current.onFocus();
  });
  expect(result.current.focused).toBe(9);

  act(() => {
    result.current.onBlur();
  });
  expect(result.current.focused).toBeNull();
});

it('prefers the hovered option over the focused one, and Escape clears both', () => {
  const { result } = setup(4);

  act(() => {
    result.current.onKeyDown(press('ArrowRight'));
    result.current.setHovered(3);
  });
  expect(result.current.shown).toBe(3);

  act(() => {
    result.current.onKeyDown(press('Escape'));
  });
  expect(result.current.focused).toBeNull();
  expect(result.current.hovered).toBeNull();
  expect(result.current.shown).toBeNull();
});

it('forgets a position that is past the end after the data shrinks', () => {
  const { result, rerender } = renderHook(({ count }) => useChartNavigation(count), {
    initialProps: { count: 5 },
  });
  act(() => {
    result.current.onKeyDown(press('End'));
  });

  rerender({ count: 2 });

  expect(result.current.focused).toBeNull();
});
