import { act, renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { useDateNavigation } from '@/features/usage/useDateNavigation';

beforeEach(() => {
  vi.useFakeTimers();
  vi.setSystemTime(new Date(2026, 9, 12, 15, 30));
});

afterEach(() => {
  vi.useRealTimers();
});

describe('useDateNavigation by day', () => {
  it('starts on today', () => {
    const { result } = renderHook(() => useDateNavigation(1));

    expect(result.current.isoDate).toBe('2026-10-12');
    expect(result.current.isCurrent).toBe(true);
  });

  it('steps back day by day and forward again until it reaches today', () => {
    const { result } = renderHook(() => useDateNavigation(1));

    act(() => {
      result.current.previous();
    });
    act(() => {
      result.current.previous();
    });
    expect(result.current.isoDate).toBe('2026-10-10');
    expect(result.current.isCurrent).toBe(false);

    act(() => {
      result.current.next();
    });
    act(() => {
      result.current.next();
    });
    expect(result.current.isoDate).toBe('2026-10-12');
    expect(result.current.isCurrent).toBe(true);
  });

  it('never goes past today', () => {
    const { result } = renderHook(() => useDateNavigation(1));

    act(() => {
      result.current.next();
    });

    expect(result.current.isoDate).toBe('2026-10-12');
  });

  it('jumps straight back to today', () => {
    const { result } = renderHook(() => useDateNavigation(1));
    act(() => {
      result.current.previous();
    });

    act(() => {
      result.current.reset();
    });

    expect(result.current.isCurrent).toBe(true);
  });

  it('follows today across midnight when it was never left', () => {
    const { result } = renderHook(() => useDateNavigation(1));

    act(() => {
      vi.setSystemTime(new Date(2026, 9, 13, 0, 1));
      vi.advanceTimersByTime(60_000);
    });

    expect(result.current.isoDate).toBe('2026-10-13');
    expect(result.current.isCurrent).toBe(true);
  });

  it('keeps a past day pinned across midnight', () => {
    const { result } = renderHook(() => useDateNavigation(1));
    act(() => {
      result.current.previous();
    });

    act(() => {
      vi.setSystemTime(new Date(2026, 9, 13, 0, 1));
      vi.advanceTimersByTime(60_000);
    });

    expect(result.current.isoDate).toBe('2026-10-11');
    expect(result.current.isCurrent).toBe(false);
  });
});

describe('useDateNavigation by week', () => {
  it('steps the end date by seven days and clamps to today', () => {
    const { result } = renderHook(() => useDateNavigation(7));

    act(() => {
      result.current.previous();
    });
    expect(result.current.isoDate).toBe('2026-10-05');

    act(() => {
      result.current.previous();
    });
    expect(result.current.isoDate).toBe('2026-09-28');

    act(() => {
      result.current.next();
    });
    act(() => {
      result.current.next();
    });
    expect(result.current.isCurrent).toBe(true);
  });

  it('clamps a partial step to today instead of overshooting', () => {
    const { result } = renderHook(() => useDateNavigation(7));
    act(() => {
      result.current.previous();
    });
    act(() => {
      vi.setSystemTime(new Date(2026, 9, 10, 9, 0));
      vi.advanceTimersByTime(60_000);
    });
    expect(result.current.isoDate).toBe('2026-10-05');

    act(() => {
      result.current.next();
    });

    expect(result.current.isCurrent).toBe(true);
    expect(result.current.isoDate).toBe('2026-10-10');
  });
});

describe('useDateNavigation memoization', () => {
  it('returns the same object across minute ticks that do not change a date', () => {
    const { result } = renderHook(() => useDateNavigation(1));
    const first = result.current;

    act(() => {
      vi.advanceTimersByTime(5 * 60_000);
    });

    expect(result.current).toBe(first);
    expect(result.current.today).toBe(first.today);
    expect(result.current.date).toBe(first.date);
  });

  it('returns a new object when the day rolls over', () => {
    const { result } = renderHook(() => useDateNavigation(1));
    const first = result.current;

    act(() => {
      vi.setSystemTime(new Date(2026, 9, 13, 0, 1));
      vi.advanceTimersByTime(60_000);
    });

    expect(result.current).not.toBe(first);
    expect(result.current.isoDate).toBe('2026-10-13');
  });
});

describe('useDateNavigation bounded by retention', () => {
  it('allows going back until the oldest day that is still kept', () => {
    const { result } = renderHook(() => useDateNavigation(1, 3));
    const walk = () => {
      act(() => {
        result.current.previous();
      });
    };

    expect(result.current.canGoPrevious).toBe(true);
    walk();
    walk();
    walk();
    expect(result.current.isoDate).toBe('2026-10-09');
    expect(result.current.canGoPrevious).toBe(false);

    walk();
    expect(result.current.isoDate).toBe('2026-10-09');
  });

  it('never steps a week past the oldest day that is still kept', () => {
    const { result } = renderHook(() => useDateNavigation(7, 20));

    act(() => {
      result.current.previous();
    });
    expect(result.current.isoDate).toBe('2026-10-05');
    expect(result.current.canGoPrevious).toBe(true);

    act(() => {
      result.current.previous();
    });
    expect(result.current.isoDate).toBe('2026-09-28');
    expect(result.current.canGoPrevious).toBe(false);

    act(() => {
      result.current.previous();
    });
    expect(result.current.isoDate).toBe('2026-09-28');
  });

  it('can always come back forward after reaching the limit', () => {
    const { result } = renderHook(() => useDateNavigation(1, 1));
    act(() => {
      result.current.previous();
    });
    expect(result.current.canGoPrevious).toBe(false);

    act(() => {
      result.current.next();
    });

    expect(result.current.isCurrent).toBe(true);
    expect(result.current.canGoPrevious).toBe(true);
  });

  it('does not limit anything while the retention period is not known', () => {
    const { result } = renderHook(() => useDateNavigation(1, null));

    for (let step = 0; step < 400; step += 1) {
      act(() => {
        result.current.previous();
      });
    }

    expect(result.current.canGoPrevious).toBe(true);
    expect(result.current.isoDate).toBe('2025-09-07');
  });

  it('follows a shorter retention period that arrives later', () => {
    const { result, rerender } = renderHook(
      ({ retention }: { retention: number | null }) => useDateNavigation(1, retention),
      { initialProps: { retention: null as number | null } },
    );
    act(() => {
      result.current.previous();
    });
    act(() => {
      result.current.previous();
    });
    expect(result.current.canGoPrevious).toBe(true);

    rerender({ retention: 2 });

    expect(result.current.canGoPrevious).toBe(false);
  });
});
