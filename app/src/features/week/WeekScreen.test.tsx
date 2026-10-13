import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn<(command: string, ...args: unknown[]) => Promise<unknown>>(),
}));

vi.mock('@/ipc/commands', () => ({ invokeCommand: invokeMock }));

import { MISSING_BROWSER_PERMISSIONS, routeCommands, weekReport } from '@/test/ipc';
import { freezeClock, renderInShell } from '@/test/shell';
import { formatMonthDay } from '@/lib/format';
import { expectAccessible } from '@/test/render';

import { WeekScreen } from './WeekScreen';

function previousWeek(): string {
  return `${formatMonthDay(new Date(2026, 8, 29))} to ${formatMonthDay(new Date(2026, 9, 5))}`;
}

beforeEach(() => {
  invokeMock.mockReset();
  freezeClock();
});

afterEach(() => {
  vi.useRealTimers();
});

function renderWeek() {
  return renderInShell(<WeekScreen />, 'week');
}

function stepper() {
  return within(screen.getByRole('group', { name: 'Choose 7 days' }));
}

function isUnavailable(button: HTMLElement): boolean {
  return button.getAttribute('aria-disabled') === 'true';
}

function title(): string {
  return screen.getByRole('heading', { level: 1 }).textContent;
}

describe('WeekScreen history limits', () => {
  it('disables Previous 7 days once another step back would pass the retention period', async () => {
    const user = userEvent.setup();
    routeCommands(invokeMock, {
      get_settings: () => ({ retentionDays: 30, launchAtLogin: false }),
    });
    renderWeek();
    await screen.findByText('Screen time, last 7 days');
    const previous = () => stepper().getByRole('button', { name: 'Previous 7 days' });
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('get_settings');
    });

    for (const endDate of ['2026-10-05', '2026-09-28', '2026-09-21', '2026-09-14']) {
      await waitFor(() => {
        expect(isUnavailable(previous())).toBe(false);
      });
      await user.click(previous());
      await waitFor(() => {
        expect(invokeMock).toHaveBeenCalledWith('get_week_report', { endDate });
      });
    }

    await waitFor(() => {
      expect(isUnavailable(previous())).toBe(true);
    });
    fireEvent.keyDown(window, { key: '[', metaKey: true });
    expect(invokeMock).not.toHaveBeenCalledWith('get_week_report', { endDate: '2026-09-07' });
  });
});

describe('WeekScreen', () => {
  it('shows the total of the last 7 days, the daily average over tracked days and the chart', async () => {
    routeCommands(invokeMock);
    renderWeek();

    expect(await screen.findByText('Screen time, last 7 days')).toBeTruthy();
    expect(screen.getByText('5h')).toBeTruthy();
    expect(screen.getByText(/Daily average/).textContent).toMatch(/^Daily average 1h15min/);
    expect(screen.getByRole('listbox', { name: 'Screen time by day' })).toBeTruthy();
    expect(title()).toBe('Last 7 days');
    expect(invokeMock).toHaveBeenCalledWith('get_week_report', { endDate: '2026-10-12' });
  });

  it('draws seven days with localized weekday ticks and highlights today', async () => {
    routeCommands(invokeMock);
    renderWeek();

    const chart = await screen.findByRole('listbox', { name: 'Screen time by day' });
    const days = within(chart).getAllByRole('option');

    expect(days).toHaveLength(7);
    expect(days.at(-1)?.getAttribute('aria-label')).toBe('Today, 0min');
    expect(days.at(-1)?.hasAttribute('data-highlight')).toBe(true);
    expect(screen.getByText('Mon')).toBeTruthy();
    expect(screen.getByText('Sun')).toBeTruthy();
  });

  it('leaves days without tracked time out of the average', async () => {
    routeCommands(invokeMock, {
      get_week_report: ({ endDate }) => weekReport(endDate, [0, 0, 0, 0, 0, 0, 6_000]),
    });
    renderWeek();

    await screen.findByText('Screen time, last 7 days');

    expect(screen.getByText(/Daily average/).textContent).toMatch(/^Daily average 1h40min/);
  });

  it('lists the top apps and sites of the 7 days', async () => {
    routeCommands(invokeMock);
    renderWeek();

    const apps = await screen.findByRole('list', { name: 'Top apps in these 7 days' });
    const sites = screen.getByRole('list', { name: 'Top sites in these 7 days' });

    expect(within(apps).getByText('Safari')).toBeTruthy();
    expect(within(sites).getByText('github.com')).toBeTruthy();
  });

  it('steps back 7 days at a time and shows the range in the toolbar', async () => {
    const user = userEvent.setup();
    routeCommands(invokeMock);
    renderWeek();
    await screen.findByText('Screen time, last 7 days');
    expect(isUnavailable(stepper().getByRole('button', { name: 'Next 7 days' }))).toBe(true);

    await user.click(stepper().getByRole('button', { name: 'Previous 7 days' }));

    await waitFor(() => {
      expect(title()).toBe(previousWeek());
    });
    expect(invokeMock).toHaveBeenCalledWith('get_week_report', { endDate: '2026-10-05' });
    expect(await screen.findByText('Screen time')).toBeTruthy();
    expect(stepper().getByRole('button', { name: 'Last 7 days' })).toBeTruthy();
  });

  it('returns with the shortcut and the reset button', async () => {
    const user = userEvent.setup();
    routeCommands(invokeMock);
    renderWeek();
    await screen.findByText('Screen time, last 7 days');

    fireEvent.keyDown(window, { key: '[', metaKey: true });
    await waitFor(() => {
      expect(title()).toBe(previousWeek());
    });
    fireEvent.keyDown(window, { key: ']', metaKey: true });
    await waitFor(() => {
      expect(title()).toBe('Last 7 days');
    });
    fireEvent.keyDown(window, { key: '[', metaKey: true });
    await waitFor(() => {
      expect(title()).toBe(previousWeek());
    });
    await user.click(stepper().getByRole('button', { name: 'Last 7 days' }));

    await waitFor(() => {
      expect(title()).toBe('Last 7 days');
    });
  });

  it('holds a skeleton while loading', () => {
    routeCommands(invokeMock, { get_week_report: () => new Promise(() => undefined) });
    renderWeek();

    expect(screen.getByRole('status', { name: 'Loading your last 7 days' })).toBeTruthy();
    expect(screen.queryByRole('listbox')).toBeNull();
  });

  it('shows an empty week, with a pointer to Settings when browser access is missing', async () => {
    const user = userEvent.setup();
    routeCommands(invokeMock, {
      get_week_report: ({ endDate }) => ({
        ...weekReport(endDate, [0, 0, 0, 0, 0, 0, 0]),
        topApps: [],
        topSites: [],
      }),
      get_permissions: () => MISSING_BROWSER_PERMISSIONS,
    });
    renderWeek();

    expect(await screen.findByText('Nothing tracked in the last 7 days')).toBeTruthy();
    expect(screen.queryByRole('listbox')).toBeNull();
    await user.click(await screen.findByRole('button', { name: 'Open Settings' }));

    await waitFor(() => {
      expect(title()).toBe('Settings');
    });
  });

  it('shows an error with a retry that recovers', async () => {
    const user = userEvent.setup();
    let failing = true;
    routeCommands(invokeMock, {
      get_week_report: ({ endDate }) => {
        if (failing) return Promise.reject(new Error('storage offline'));
        return weekReport(endDate, [3_600, 0, 0, 0, 0, 0, 0]);
      },
    });
    renderWeek();

    expect(await screen.findByRole('alert')).toBeTruthy();
    failing = false;
    await user.click(screen.getByRole('button', { name: 'Try again' }));

    expect(await screen.findByText('Screen time, last 7 days')).toBeTruthy();
  });

  it('has no accessibility violations', async () => {
    routeCommands(invokeMock);
    const { container } = renderWeek();
    await screen.findByRole('list', { name: 'Top apps in these 7 days' });

    await expectAccessible(container, true);
  });
});
