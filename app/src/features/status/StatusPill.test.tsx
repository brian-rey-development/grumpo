import { act, screen } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn<(command: string, ...args: unknown[]) => Promise<unknown>>(),
}));

vi.mock('@/ipc/commands', () => ({ invokeCommand: invokeMock }));

import type { FocusSnapshot, FocusState, UnlockWait } from '@/features/status/focusState';
import { StatusPill } from '@/features/status/StatusPill';
import { routeCommands } from '@/test/ipc';
import { expectAccessible, renderWithProviders } from '@/test/render';

beforeEach(() => {
  invokeMock.mockReset();
  routeCommands(invokeMock);
  vi.useFakeTimers();
  vi.setSystemTime(new Date(2026, 9, 12, 12, 0, 0));
});

afterEach(() => {
  vi.useRealTimers();
});

function snapshot(state: FocusState, unlockWait: UnlockWait | null = null): FocusSnapshot {
  return { state, unlockWait };
}

it('shows when the focus block ends', () => {
  const until = new Date(2026, 9, 12, 19, 0);
  renderWithProviders(<StatusPill snapshot={snapshot({ kind: 'focus', until })} />);

  expect(screen.getByText('Focus on until 19:00')).toBeTruthy();
});

it('writes the end time in the hour cycle of the Mac once it is known', async () => {
  vi.useRealTimers();
  routeCommands(invokeMock, { get_clock_style: () => 'h12' });
  const until = new Date(2026, 9, 12, 19, 0);
  renderWithProviders(<StatusPill snapshot={snapshot({ kind: 'focus', until })} />);
  expect(screen.getByText('Focus on until 19:00')).toBeTruthy();

  const label = await screen.findByText(/^Focus on until 7:00\spm$/i);

  expect(label).toBeTruthy();
});

it('falls back to a plain focus label without an end time', () => {
  renderWithProviders(<StatusPill snapshot={snapshot({ kind: 'focus', until: null })} />);

  expect(screen.getByText('Focus on')).toBeTruthy();
});

it('counts down while unlocked, with the countdown in the numeric font', () => {
  const until = new Date(2026, 9, 12, 12, 15, 0);
  renderWithProviders(<StatusPill snapshot={snapshot({ kind: 'unlocked', until })} />);
  const countdown = screen.getByText('15:00');
  expect(countdown.parentElement?.textContent).toBe('Unlocked, 15:00 left');
  expect(countdown.className).toContain('numeric');

  act(() => {
    vi.advanceTimersByTime(5_000);
  });

  expect(screen.getByText('14:55')).toBeTruthy();
});

it('counts down to the end of an unlock wait, then says it is ready', () => {
  const readyAt = new Date(2026, 9, 12, 12, 4, 12);
  renderWithProviders(
    <StatusPill
      snapshot={snapshot({ kind: 'focus', until: null }, { kind: 'coolingDown', readyAt })}
    />,
  );
  const countdown = screen.getByText('4:12');
  expect(countdown.parentElement?.textContent).toBe('Unlock in 4:12');
  expect(countdown.className).toContain('numeric');

  act(() => {
    vi.advanceTimersByTime(252_000);
  });

  expect(screen.getByText('Unlock ready')).toBeTruthy();
});

it('names the time to confirm by once the unlock is ready', () => {
  const confirmBy = new Date(2026, 9, 12, 12, 5, 0);
  renderWithProviders(
    <StatusPill
      snapshot={snapshot({ kind: 'focus', until: null }, { kind: 'ready', confirmBy })}
    />,
  );

  expect(screen.getByText('Unlock ready, confirm by 12:05')).toBeTruthy();
});

it.each<[FocusState, string]>([
  [{ kind: 'limit', targets: [] }, 'Daily limit reached'],
  [{ kind: 'free' }, 'No block active'],
  [{ kind: 'unavailable' }, 'Grumpo service not running'],
])('labels the %j state', (state, label) => {
  renderWithProviders(<StatusPill snapshot={snapshot(state)} />);

  expect(screen.getByText(label)).toBeTruthy();
});

it('never calls the service a daemon in the interface', () => {
  renderWithProviders(<StatusPill snapshot={snapshot({ kind: 'unavailable' })} />);

  expect(screen.queryByText(/daemon/i)).toBeNull();
});

it('has no accessibility violations in any state', async () => {
  vi.useRealTimers();
  const soon = (ms: number) => new Date(Date.now() + ms);
  const { container } = renderWithProviders(
    <>
      <StatusPill snapshot={snapshot({ kind: 'focus', until: new Date(2026, 9, 12, 19, 0) })} />
      <StatusPill snapshot={snapshot({ kind: 'unlocked', until: soon(900_000) })} />
      <StatusPill
        snapshot={snapshot(
          { kind: 'focus', until: null },
          { kind: 'coolingDown', readyAt: soon(252_000) },
        )}
      />
      <StatusPill
        snapshot={snapshot({ kind: 'focus', until: null }, { kind: 'ready', confirmBy: soon(1) })}
      />
      <StatusPill snapshot={snapshot({ kind: 'limit', targets: [] })} />
      <StatusPill snapshot={snapshot({ kind: 'free' })} />
      <StatusPill snapshot={snapshot({ kind: 'unavailable' })} />
    </>,
  );

  await expectAccessible(container);
});
