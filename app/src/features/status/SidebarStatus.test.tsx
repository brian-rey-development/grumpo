import { act, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn<(command: string, ...args: unknown[]) => Promise<unknown>>(),
}));

vi.mock('@/ipc/commands', () => ({ invokeCommand: invokeMock }));

import { SidebarStatus } from '@/features/status/SidebarStatus';
import { connectedStatus, NOT_INSTALLED, routeCommands, unixSeconds } from '@/test/ipc';
import type { FocusStatus } from '@/ipc/generated/FocusStatus';
import { formatClock } from '@/lib/format';
import { MS_PER_SECOND } from '@/lib/time';
import { expectAccessible, renderWithProviders } from '@/test/render';

const NOW = new Date(2026, 9, 12, 12, 0, 0);

function after(seconds: number): Date {
  return new Date(NOW.getTime() + seconds * MS_PER_SECOND);
}

beforeEach(() => {
  invokeMock.mockReset();
  vi.useFakeTimers({ toFake: ['Date'] });
  vi.setSystemTime(NOW);
});

afterEach(() => {
  vi.useRealTimers();
});

function renderStatus(status: FocusStatus) {
  routeCommands(invokeMock, { get_focus_status: () => status });
  const onOpenUnlock = vi.fn();
  const onOpenSettings = vi.fn();
  const view = renderWithProviders(
    <SidebarStatus onOpenUnlock={onOpenUnlock} onOpenSettings={onOpenSettings} />,
  );
  return { ...view, onOpenUnlock, onOpenSettings };
}

const UNLOCK_HINT = 'Opens unlock options';
const SETTINGS_HINT = 'Opens Settings';

interface Variant {
  name: string;
  status: FocusStatus;
  buttonName: string;
  hint: string;
  opens: 'unlock' | 'settings';
}

const VARIANTS: Variant[] = [
  {
    name: 'a schedule block with an end time',
    status: connectedStatus({
      active: true,
      reason: 'schedule',
      activeUntil: new Date(2026, 9, 12, 19, 0),
    }),
    buttonName: 'Focus on until 19:00',
    hint: UNLOCK_HINT,
    opens: 'unlock',
  },
  {
    name: 'a block without an end time',
    status: connectedStatus({ active: true, reason: 'both' }),
    buttonName: 'Focus on',
    hint: UNLOCK_HINT,
    opens: 'unlock',
  },
  {
    name: 'an exhausted daily limit',
    status: connectedStatus({
      active: true,
      reason: 'limit',
      exhausted: [{ kind: 'site', domain: 'youtube.com' }],
    }),
    buttonName: 'Daily limit reached',
    hint: UNLOCK_HINT,
    opens: 'unlock',
  },
  {
    name: 'an unlock cooldown',
    status: connectedStatus({
      active: true,
      reason: 'schedule',
      phase: { kind: 'coolingDown', readyAt: unixSeconds(after(252)), minutes: 5 },
    }),
    buttonName: 'Unlock in 4:12',
    hint: UNLOCK_HINT,
    opens: 'unlock',
  },
  {
    name: 'an unlock ready to confirm',
    status: connectedStatus({
      active: true,
      reason: 'schedule',
      phase: {
        kind: 'ready',
        readyAt: unixSeconds(after(-30)),
        expiresAt: unixSeconds(after(270)),
      },
    }),
    buttonName: `Unlock ready, confirm by ${formatClock(after(270))}`,
    hint: UNLOCK_HINT,
    opens: 'unlock',
  },
  {
    name: 'an active unlock',
    status: connectedStatus({ phase: { kind: 'unlocked', until: unixSeconds(after(900)) } }),
    buttonName: 'Unlocked, 15:00 left',
    hint: UNLOCK_HINT,
    opens: 'unlock',
  },
  {
    name: 'a service that is not running',
    status: NOT_INSTALLED,
    buttonName: 'Grumpo service not running',
    hint: SETTINGS_HINT,
    opens: 'settings',
  },
];

describe('SidebarStatus', () => {
  it.each(VARIANTS)('maps $name to a button that opens the right place', async (variant) => {
    const user = userEvent.setup();
    const { onOpenUnlock, onOpenSettings } = renderStatus(variant.status);

    await user.click(
      await screen.findByRole('button', { name: variant.buttonName, description: variant.hint }),
    );

    expect(onOpenUnlock).toHaveBeenCalledTimes(variant.opens === 'unlock' ? 1 : 0);
    expect(onOpenSettings).toHaveBeenCalledTimes(variant.opens === 'settings' ? 1 : 0);
  });

  it('shows a quiet pill without a button when nothing is blocked', async () => {
    renderStatus(connectedStatus({ active: false }));

    expect(await screen.findByText('No block active')).toBeTruthy();
    expect(screen.queryByRole('button')).toBeNull();
  });

  it('treats an unlock that already ran out as no longer unlocked', async () => {
    renderStatus(
      connectedStatus({
        active: false,
        phase: { kind: 'unlocked', until: unixSeconds(after(-5)) },
      }),
    );

    expect(await screen.findByText('No block active')).toBeTruthy();
  });

  it('keeps the block visible while a cooldown runs', async () => {
    renderStatus(
      connectedStatus({
        active: true,
        reason: 'schedule',
        phase: { kind: 'coolingDown', readyAt: unixSeconds(after(60)), minutes: 5 },
      }),
    );

    expect(await screen.findByText('Unlock in', { exact: false })).toBeTruthy();
    expect(screen.queryByText('Focus on')).toBeNull();
  });

  it('reports the service as not running when the status cannot be read', async () => {
    invokeMock.mockRejectedValue(new Error('boom'));
    renderWithProviders(<SidebarStatus onOpenUnlock={vi.fn()} onOpenSettings={vi.fn()} />);

    expect(
      await screen.findByRole('button', {
        name: 'Grumpo service not running',
        description: SETTINGS_HINT,
      }),
    ).toBeTruthy();
  });

  it('holds a placeholder of the same height while the status loads', () => {
    invokeMock.mockReturnValue(new Promise(() => undefined));
    renderWithProviders(<SidebarStatus onOpenUnlock={vi.fn()} onOpenSettings={vi.fn()} />);

    expect(screen.getByRole('status', { name: 'Loading focus status' })).toBeTruthy();
    expect(screen.queryByRole('button')).toBeNull();
  });

  it('has no accessibility violations', async () => {
    const { container } = renderStatus(VARIANTS[3]?.status ?? NOT_INSTALLED);
    await screen.findByRole('button');

    await expectAccessible(container);
  });
});

describe('SidebarStatus countdown', () => {
  it('ticks every second while a cooldown is running', async () => {
    vi.useRealTimers();
    vi.useFakeTimers();
    vi.setSystemTime(NOW);
    routeCommands(invokeMock, {
      get_focus_status: () =>
        connectedStatus({
          active: true,
          reason: 'schedule',
          phase: { kind: 'coolingDown', readyAt: unixSeconds(after(252)), minutes: 5 },
        }),
    });
    renderWithProviders(<SidebarStatus onOpenUnlock={vi.fn()} onOpenSettings={vi.fn()} />);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(10);
    });
    expect(screen.getByText('4:12', { exact: false })).toBeTruthy();

    await act(async () => {
      await vi.advanceTimersByTimeAsync(12_000);
    });

    expect(screen.getByText('4:00', { exact: false })).toBeTruthy();
  });
});
