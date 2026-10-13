import { act, fireEvent, render, renderHook, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { Toast } from '@/components/Toast/Toast';
import { useToast } from '@/components/Toast/useToast';
import { mockMatchMedia } from '@/test/matchMedia';
import { expectAccessible } from '@/test/render';

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

it('keeps an empty polite live region before anything is shown', () => {
  render(<Toast dismissLabel="Dismiss" toast={null} onDismiss={() => undefined} />);

  const region = document.body.querySelector('[aria-live="polite"]');
  expect(region).not.toBeNull();
  expect(region?.textContent).toBe('');
});

it('shows the text without moving focus', () => {
  render(
    <Toast
      dismissLabel="Dismiss"
      toast={{ id: 1, text: 'Exported to Downloads' }}
      onDismiss={() => undefined}
    />,
  );

  expect(screen.getByText('Exported to Downloads')).toBeTruthy();
  expect(document.activeElement).toBe(document.body);
});

it('dismisses itself after the duration', () => {
  vi.useFakeTimers();
  const onDismiss = vi.fn();
  render(
    <Toast
      dismissLabel="Dismiss"
      toast={{ id: 1, text: 'Saved' }}
      durationMs={3_000}
      onDismiss={onDismiss}
    />,
  );

  act(() => {
    vi.advanceTimersByTime(2_999);
  });
  expect(onDismiss).not.toHaveBeenCalled();
  act(() => {
    vi.advanceTimersByTime(1);
  });
  expect(onDismiss).toHaveBeenCalledOnce();
});

it('restarts the timer when a newer message replaces it', () => {
  vi.useFakeTimers();
  const onDismiss = vi.fn();
  const { rerender } = render(
    <Toast
      dismissLabel="Dismiss"
      toast={{ id: 1, text: 'One' }}
      durationMs={3_000}
      onDismiss={onDismiss}
    />,
  );

  act(() => {
    vi.advanceTimersByTime(2_000);
  });
  rerender(
    <Toast
      dismissLabel="Dismiss"
      toast={{ id: 2, text: 'Two' }}
      durationMs={3_000}
      onDismiss={onDismiss}
    />,
  );
  act(() => {
    vi.advanceTimersByTime(2_000);
  });

  expect(onDismiss).not.toHaveBeenCalled();
});

describe('pausing', () => {
  const TOAST = { id: 1, text: 'Saved' };

  it('holds on while the pointer is over it and counts down again once it leaves', () => {
    vi.useFakeTimers();
    const onDismiss = vi.fn();
    render(<Toast dismissLabel="Dismiss" toast={TOAST} durationMs={3_000} onDismiss={onDismiss} />);
    const toast = screen.getByText('Saved').parentElement;
    if (toast === null) throw new Error('toast not found');

    fireEvent.pointerEnter(toast);
    act(() => {
      vi.advanceTimersByTime(10_000);
    });
    expect(onDismiss).not.toHaveBeenCalled();

    fireEvent.pointerLeave(toast);
    act(() => {
      vi.advanceTimersByTime(2_999);
    });
    expect(onDismiss).not.toHaveBeenCalled();
    act(() => {
      vi.advanceTimersByTime(1);
    });
    expect(onDismiss).toHaveBeenCalledOnce();
  });

  it('holds on while a control inside has the keyboard focus', () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    const onDismiss = vi.fn();
    render(<Toast dismissLabel="Dismiss" toast={TOAST} durationMs={3_000} onDismiss={onDismiss} />);

    act(() => {
      screen.getByRole('button', { name: 'Dismiss' }).focus();
    });
    act(() => {
      vi.advanceTimersByTime(10_000);
    });
    expect(onDismiss).not.toHaveBeenCalled();

    act(() => {
      screen.getByRole('button', { name: 'Dismiss' }).blur();
    });
    act(() => {
      vi.advanceTimersByTime(3_000);
    });
    expect(onDismiss).toHaveBeenCalledOnce();
  });

  it('does not carry the pause over to the next message', () => {
    vi.useFakeTimers();
    const onDismiss = vi.fn();
    const { rerender } = render(
      <Toast dismissLabel="Dismiss" toast={TOAST} durationMs={3_000} onDismiss={onDismiss} />,
    );
    const toast = screen.getByText('Saved').parentElement;
    if (toast === null) throw new Error('toast not found');
    fireEvent.pointerEnter(toast);

    rerender(
      <Toast
        dismissLabel="Dismiss"
        toast={{ id: 2, text: 'Next' }}
        durationMs={3_000}
        onDismiss={onDismiss}
      />,
    );
    act(() => {
      vi.advanceTimersByTime(3_000);
    });

    expect(onDismiss).toHaveBeenCalledOnce();
  });
});

it('can be dismissed with the keyboard', async () => {
  const onDismiss = vi.fn();
  const user = userEvent.setup();
  render(<Toast dismissLabel="Dismiss" toast={{ id: 1, text: 'Saved' }} onDismiss={onDismiss} />);

  await user.tab();
  await user.keyboard('{Enter}');

  expect(onDismiss).toHaveBeenCalledOnce();
});

it('goes away at once when motion is reduced', async () => {
  mockMatchMedia({ reducedMotion: true });
  const { rerender } = render(
    <Toast dismissLabel="Dismiss" toast={{ id: 1, text: 'Saved' }} onDismiss={() => undefined} />,
  );

  rerender(<Toast dismissLabel="Dismiss" toast={null} onDismiss={() => undefined} />);

  await waitFor(() => {
    expect(screen.queryByText('Saved')).toBeNull();
  });
});

it('tracks the current toast in useToast', () => {
  const { result } = renderHook(() => useToast());
  expect(result.current.toast).toBeNull();

  act(() => {
    result.current.show('First');
  });
  const first = result.current.toast;
  act(() => {
    result.current.show('Second');
  });

  expect(first?.text).toBe('First');
  expect(result.current.toast?.text).toBe('Second');
  expect(result.current.toast?.id).not.toBe(first?.id);

  act(() => {
    result.current.dismiss();
  });
  expect(result.current.toast).toBeNull();
});

it('has no accessibility violations', async () => {
  render(
    <Toast
      dismissLabel="Dismiss"
      toast={{ id: 1, text: 'Exported to Downloads' }}
      onDismiss={() => undefined}
    />,
  );

  await expectAccessible(document.body);
});
