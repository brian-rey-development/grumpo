import { waitFor } from '@testing-library/react';
import { beforeEach, expect, it, vi } from 'vitest';

const { subscribeMock } = vi.hoisted(() => ({
  subscribeMock:
    vi.fn<(event: string, handler: (payload: unknown) => void) => Promise<() => void>>(),
}));

vi.mock('@/ipc/events', () => ({ subscribe: subscribeMock }));

import { useBackendEvent } from '@/ipc/useBackendEvent';
import { renderWithProviders } from '@/test/render';

beforeEach(() => {
  subscribeMock.mockReset();
});

function Listener({
  onEvent,
  onSubscribed,
}: {
  onEvent: (payload: null) => void;
  onSubscribed?: () => void;
}) {
  useBackendEvent('usage-changed', onEvent, onSubscribed);
  return null;
}

it('forwards the payload of each event to the handler', async () => {
  let emit: (payload: unknown) => void = () => undefined;
  subscribeMock.mockImplementation((_event, handler) => {
    emit = handler;
    return Promise.resolve(vi.fn());
  });
  const onEvent = vi.fn();

  renderWithProviders(<Listener onEvent={onEvent} />);
  await waitFor(() => {
    expect(subscribeMock).toHaveBeenCalledWith('usage-changed', expect.any(Function));
  });
  emit(null);

  expect(onEvent).toHaveBeenCalledWith(null);
});

it('calls onSubscribed exactly once after the listener is live', async () => {
  subscribeMock.mockResolvedValue(vi.fn());
  const onSubscribed = vi.fn();

  const { rerender } = renderWithProviders(
    <Listener onEvent={vi.fn()} onSubscribed={onSubscribed} />,
  );
  await waitFor(() => {
    expect(onSubscribed).toHaveBeenCalledOnce();
  });
  rerender(<Listener onEvent={vi.fn()} onSubscribed={onSubscribed} />);

  expect(onSubscribed).toHaveBeenCalledOnce();
});

it('unsubscribes on unmount', async () => {
  const stop = vi.fn();
  subscribeMock.mockResolvedValue(stop);
  const onSubscribed = vi.fn();

  const { unmount } = renderWithProviders(
    <Listener onEvent={vi.fn()} onSubscribed={onSubscribed} />,
  );
  await waitFor(() => {
    expect(onSubscribed).toHaveBeenCalled();
  });
  unmount();

  expect(stop).toHaveBeenCalledOnce();
});

it('unsubscribes without resyncing when unmounted before the subscription resolves', async () => {
  const stop = vi.fn();
  const onSubscribed = vi.fn();
  let resolve: (value: () => void) => void = () => undefined;
  subscribeMock.mockReturnValue(
    new Promise<() => void>((done) => {
      resolve = done;
    }),
  );

  const { unmount } = renderWithProviders(
    <Listener onEvent={vi.fn()} onSubscribed={onSubscribed} />,
  );
  unmount();
  resolve(stop);

  await waitFor(() => {
    expect(stop).toHaveBeenCalledOnce();
  });
  expect(onSubscribed).not.toHaveBeenCalled();
});

it('logs and survives a failed subscription', async () => {
  const error = vi.spyOn(console, 'error').mockImplementation(() => undefined);
  subscribeMock.mockRejectedValue(new Error('no ipc'));

  renderWithProviders(<Listener onEvent={vi.fn()} />);

  await waitFor(() => {
    expect(error).toHaveBeenCalled();
  });
  error.mockRestore();
});
