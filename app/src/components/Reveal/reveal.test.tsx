import { act, render, renderHook } from '@testing-library/react';
import type { ReactNode } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { REVEAL_SETTLE_MS, RevealProvider, useRevealOnce } from '@/components/Reveal/Reveal';

function setHidden(hidden: boolean): void {
  vi.spyOn(document, 'hidden', 'get').mockReturnValue(hidden);
}

afterEach(() => {
  vi.useRealTimers();
  vi.restoreAllMocks();
});

function wrapper({ children }: { children: ReactNode }) {
  return <RevealProvider>{children}</RevealProvider>;
}

describe('useRevealOnce', () => {
  it('waits for data, then plays once', () => {
    const { result, rerender } = renderHook(
      ({ ready }: { ready: boolean }) => useRevealOnce('data', ready),
      { wrapper, initialProps: { ready: false } },
    );
    expect(result.current).toBe('waiting');

    rerender({ ready: true });

    expect(result.current).toBe('play');
  });

  it('is idle for a dataset that was shown before', () => {
    vi.useFakeTimers();
    let latest = '';
    function Probe() {
      latest = useRevealOnce('again', true);
      return null;
    }
    const { rerender } = render(
      <RevealProvider>
        <Probe />
      </RevealProvider>,
    );
    expect(latest).toBe('play');

    act(() => {
      vi.advanceTimersByTime(REVEAL_SETTLE_MS);
    });
    rerender(<RevealProvider>{null}</RevealProvider>);
    rerender(
      <RevealProvider>
        <Probe />
      </RevealProvider>,
    );

    expect(latest).toBe('idle');
  });

  it('plays again when the view is replaced before its entrance could finish', () => {
    vi.useFakeTimers();
    let latest = '';
    function Probe() {
      latest = useRevealOnce('quick', true);
      return null;
    }
    const { rerender } = render(
      <RevealProvider>
        <Probe />
      </RevealProvider>,
    );

    rerender(<RevealProvider>{null}</RevealProvider>);
    rerender(
      <RevealProvider>
        <Probe />
      </RevealProvider>,
    );

    expect(latest).toBe('play');
  });

  it('skips the entrance for good when the window is hidden as the data arrives', () => {
    setHidden(true);
    const { result, rerender } = renderHook(
      ({ ready }: { ready: boolean }) => useRevealOnce('hidden-data', ready),
      { wrapper, initialProps: { ready: true } },
    );
    expect(result.current).toBe('idle');

    setHidden(false);
    rerender({ ready: true });

    expect(result.current).toBe('idle');
  });

  it('plays when the window is visible as the data arrives, even if it is hidden afterwards', () => {
    const { result, rerender } = renderHook(
      ({ ready }: { ready: boolean }) => useRevealOnce('late', ready),
      { wrapper, initialProps: { ready: false } },
    );
    rerender({ ready: true });

    setHidden(true);
    rerender({ ready: true });

    expect(result.current).toBe('play');
  });
});

it('refuses to run outside a RevealProvider', () => {
  expect(() => renderHook(() => useRevealOnce('orphan', true))).toThrow(/RevealProvider/);
});
