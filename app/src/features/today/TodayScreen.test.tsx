import { act, fireEvent, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn<(command: string, ...args: unknown[]) => Promise<unknown>>(),
}));

vi.mock('@/ipc/commands', () => ({ invokeCommand: invokeMock }));

import {
  connectedStatus,
  emptyDayReport,
  manyApps,
  limitUsage,
  MISSING_BROWSER_PERMISSIONS,
  NOT_INSTALLED,
  populatedDayReport,
  routeCommands,
} from '@/test/ipc';
import { freezeClock, renderInShell } from '@/test/shell';
import { expectAccessible } from '@/test/render';

import { TodayScreen } from './TodayScreen';

beforeEach(() => {
  invokeMock.mockReset();
  freezeClock();
});

afterEach(() => {
  vi.useRealTimers();
});

function renderToday() {
  return renderInShell(<TodayScreen />);
}

function isUnavailable(button: HTMLElement): boolean {
  return button.getAttribute('aria-disabled') === 'true';
}

function stepper() {
  return within(screen.getByRole('group', { name: 'Choose a day' }));
}

function title(): string {
  return screen.getByRole('heading', { level: 1 }).textContent;
}

describe('TodayScreen loaded', () => {
  it('shows the banner, the total, the day chart and both lists', async () => {
    routeCommands(invokeMock, {
      get_focus_status: () =>
        connectedStatus({
          active: true,
          reason: 'schedule',
          activeUntil: new Date(2026, 9, 12, 19, 0),
        }),
    });
    renderToday();

    expect(await screen.findByText('1h 40min')).toBeTruthy();
    expect(screen.getByText('Screen time today')).toBeTruthy();
    expect(
      await screen.findByRole('heading', { level: 2, name: 'On duty until 19:00' }),
    ).toBeTruthy();
    expect(screen.getByText('Guarding 2 sites and 1 app.')).toBeTruthy();
    expect(screen.getByRole('listbox', { name: 'Screen time by hour' })).toBeTruthy();
    expect(
      within(screen.getByRole('list', { name: 'Apps used this day' })).getByText('Safari'),
    ).toBeTruthy();
    expect(
      within(screen.getByRole('list', { name: 'Sites visited this day' })).getByText('github.com'),
    ).toBeTruthy();
    expect(title()).toBe('Today');
  });

  it('marks the current hour in the chart', async () => {
    routeCommands(invokeMock);
    renderToday();

    await screen.findByRole('listbox', { name: 'Screen time by hour' });

    const column = screen.getByRole('option', { name: /^15:00 to 16:00/ });
    expect(column.getAttribute('data-highlight')).toBe('bar');
  });

  it('says when a daily limit is used up, in the banner', async () => {
    routeCommands(invokeMock, {
      get_focus_status: () =>
        connectedStatus({
          active: true,
          reason: 'limit',
          exhausted: [{ kind: 'site', domain: 'youtube.com' }],
        }),
    });
    renderToday();

    expect(
      await screen.findByRole('heading', { level: 2, name: 'youtube.com is done for today.' }),
    ).toBeTruthy();
    expect(screen.getByText('Your daily limit is used up.')).toBeTruthy();
  });

  it('says when the next shift starts while Grumpo is off duty', async () => {
    routeCommands(invokeMock, { get_focus_status: () => connectedStatus({ active: false }) });
    renderToday();

    expect(await screen.findByRole('heading', { level: 2, name: 'Off duty' })).toBeTruthy();
    expect(screen.getByText('Next shift starts tomorrow at 09:00.')).toBeTruthy();
  });

  it('offers a way to Settings when the focus service is not running', async () => {
    const user = userEvent.setup();
    routeCommands(invokeMock, { get_focus_status: () => NOT_INSTALLED });
    renderToday();

    await screen.findByRole('heading', { level: 2, name: 'Grumpo service not running' });
    await user.click(screen.getByRole('button', { name: 'Open Settings' }));

    await waitFor(() => {
      expect(title()).toBe('Settings');
    });
  });

  it('follows new data without a reload', async () => {
    let total = 6_000;
    routeCommands(invokeMock, {
      get_day_report: ({ date }) => ({ ...populatedDayReport(date), totalSeconds: total }),
    });
    const { client } = renderToday();
    await screen.findByText('1h 40min');

    total = 7_200;
    await act(async () => {
      await client.invalidateQueries();
    });

    await waitFor(() => {
      expect(screen.getByText('2h')).toBeTruthy();
    });
  });

  it('has no accessibility violations', async () => {
    routeCommands(invokeMock, {
      get_limit_usage: () => [
        {
          target: { kind: 'site', domain: 'youtube.com' },
          usedSeconds: 2_280,
          limitSeconds: 3_600,
          reached: false,
        },
      ],
    });
    const { container } = renderToday();
    await screen.findByRole('list', { name: 'Sites visited this day' });
    await screen.findByRole('progressbar');

    await expectAccessible(container, true);
  });
});

describe('TodayScreen focus times on the chart', () => {
  const FOCUS_NOTE = 'Shaded hours are when focus time was scheduled.';

  it('explains the shaded hours and names them for a screen reader', async () => {
    routeCommands(invokeMock);
    renderToday();

    expect(await screen.findByText(FOCUS_NOTE)).toBeTruthy();
    expect(screen.getByText('Focus time was scheduled 09:00 to 12:00.')).toBeTruthy();
  });

  it('leaves the note out when the day has no focus times, as past days have none', async () => {
    const user = userEvent.setup();
    routeCommands(invokeMock, {
      get_day_report: ({ date }) => ({
        ...populatedDayReport(date),
        focusWindows: date === '2026-10-12' ? populatedDayReport(date).focusWindows : [],
      }),
    });
    renderToday();
    await screen.findByText(FOCUS_NOTE);

    await user.click(screen.getByRole('button', { name: 'Previous day' }));

    await waitFor(() => {
      expect(screen.queryByText(FOCUS_NOTE)).toBeNull();
    });
    expect(screen.getByRole('listbox', { name: 'Screen time by hour' })).toBeTruthy();
  });
});

describe('TodayScreen daily limits', () => {
  it('shows a limit on the row of the site it limits', async () => {
    routeCommands(invokeMock, {
      get_limit_usage: () => [
        limitUsage({
          target: { kind: 'site', domain: 'github.com' },
          usedSeconds: 600,
          limitSeconds: 3_600,
        }),
      ],
    });
    renderToday();

    const meter = await screen.findByRole('progressbar', { name: 'github.com' });
    expect(meter.getAttribute('aria-valuetext')).toBe('10min of 1h');
    expect(screen.getByText('50min left of 1h')).toBeTruthy();
  });

  it('says so when a limit is reached', async () => {
    routeCommands(invokeMock, {
      get_limit_usage: () => [
        limitUsage({
          target: { kind: 'site', domain: 'github.com' },
          usedSeconds: 3_600,
          limitSeconds: 3_600,
          reached: true,
        }),
      ],
    });
    renderToday();

    expect(await screen.findByText('Limit reached')).toBeTruthy();
  });
});

describe('TodayScreen apps and sites', () => {
  it('caps a list at eight rows and discloses the rest on request', async () => {
    const user = userEvent.setup();
    routeCommands(invokeMock, {
      get_day_report: ({ date }) => ({ ...populatedDayReport(date), topApps: manyApps(12) }),
    });
    renderToday();
    const apps = await screen.findByRole('list', { name: 'Apps used this day' });
    expect(within(apps).getAllByRole('listitem')).toHaveLength(8);

    await user.click(screen.getByRole('button', { name: 'Show all 12' }));
    expect(within(apps).getAllByRole('listitem')).toHaveLength(12);
    expect(screen.getByRole('button', { name: 'Show fewer' }).getAttribute('aria-expanded')).toBe(
      'true',
    );

    await user.click(screen.getByRole('button', { name: 'Show fewer' }));
    await waitFor(() => {
      expect(within(apps).getAllByRole('listitem')).toHaveLength(8);
    });
  });

  it('offers no disclosure for short lists', async () => {
    routeCommands(invokeMock);
    renderToday();
    await screen.findByRole('list', { name: 'Apps used this day' });

    expect(screen.queryByRole('button', { name: /Show all/ })).toBeNull();
  });

  it('uses the real app icon once it loads', async () => {
    routeCommands(invokeMock, { get_app_icon: () => 'data:image/png;base64,AAAA' });
    const { container } = renderToday();
    await screen.findByRole('list', { name: 'Apps used this day' });

    await waitFor(() => {
      expect(container.querySelector('img[src="data:image/png;base64,AAAA"]')).not.toBeNull();
    });
  });

  it('explains missing sites when browser access was not granted', async () => {
    routeCommands(invokeMock, {
      get_permissions: () => MISSING_BROWSER_PERMISSIONS,
      get_day_report: ({ date }) => ({ ...populatedDayReport(date), topSites: [] }),
    });
    renderToday();

    expect(await screen.findByText('Allow browser access in Settings to see sites.')).toBeTruthy();
  });
});

describe('TodayScreen browsing days', () => {
  it('steps to the previous day, shows its date and offers a way back', async () => {
    const user = userEvent.setup();
    routeCommands(invokeMock);
    renderToday();
    await screen.findByText('1h 40min');
    expect(isUnavailable(screen.getByRole('button', { name: 'Next day' }))).toBe(true);
    expect(stepper().queryByRole('button', { name: 'Today' })).toBeNull();

    await user.click(screen.getByRole('button', { name: 'Previous day' }));

    await waitFor(() => {
      expect(title()).toBe('Yesterday');
    });
    expect(invokeMock).toHaveBeenCalledWith('get_day_report', { date: '2026-10-11' });
    expect(await screen.findByText(/Sunday,? 11 October 2026/)).toBeTruthy();
    expect(screen.queryByRole('heading', { level: 2, name: 'Off duty' })).toBeNull();
    expect(screen.queryByRole('region', { name: 'Focus status' })).toBeNull();
    expect(screen.getByText('Screen time')).toBeTruthy();
    expect(
      screen.getByRole('option', { name: /^15:00 to 16:00/ }).hasAttribute('data-highlight'),
    ).toBe(false);
    expect(isUnavailable(screen.getByRole('button', { name: 'Next day' }))).toBe(false);
  });

  it('labels older days with their date in the toolbar', async () => {
    const user = userEvent.setup();
    routeCommands(invokeMock);
    renderToday();
    await screen.findByText('1h 40min');

    await user.click(screen.getByRole('button', { name: 'Previous day' }));
    await user.click(screen.getByRole('button', { name: 'Previous day' }));

    await waitFor(() => {
      expect(title()).toBe('Sat 10 Oct');
    });
  });

  it('returns to today with the Today button and with Cmd+] one day at a time', async () => {
    const user = userEvent.setup();
    routeCommands(invokeMock);
    renderToday();
    await screen.findByText('1h 40min');
    await user.click(screen.getByRole('button', { name: 'Previous day' }));
    await user.click(screen.getByRole('button', { name: 'Previous day' }));

    fireEvent.keyDown(window, { key: ']', metaKey: true });
    await waitFor(() => {
      expect(title()).toBe('Yesterday');
    });
    await user.click(stepper().getByRole('button', { name: 'Today' }));

    await waitFor(() => {
      expect(title()).toBe('Today');
    });
    expect(stepper().queryByRole('button', { name: 'Today' })).toBeNull();
  });

  it('steps back with Cmd+[', async () => {
    routeCommands(invokeMock);
    renderToday();
    await screen.findByText('1h 40min');

    fireEvent.keyDown(window, { key: '[', metaKey: true });

    await waitFor(() => {
      expect(title()).toBe('Yesterday');
    });
  });

  it('keeps the previous day on show, dimmed and busy, until the next one arrives', async () => {
    const user = userEvent.setup();
    let release: (() => void) | undefined;
    routeCommands(invokeMock, {
      get_day_report: async ({ date }) => {
        if (date === '2026-10-11') await new Promise<void>((resolve) => (release = resolve));
        return populatedDayReport(date);
      },
    });
    const { container } = renderToday();
    await screen.findByText('1h 40min');

    await user.click(screen.getByRole('button', { name: 'Previous day' }));

    await waitFor(() => {
      expect(container.querySelector('[aria-busy="true"]')).not.toBeNull();
    });
    expect(screen.getByText('1h 40min')).toBeTruthy();
    await act(async () => {
      release?.();
      await Promise.resolve();
    });
    await waitFor(() => {
      expect(container.querySelector('[aria-busy="true"]')).toBeNull();
    });
  });

  it('shows no limits when looking at a past day', async () => {
    const user = userEvent.setup();
    routeCommands(invokeMock, {
      get_limit_usage: () => [
        limitUsage({
          target: { kind: 'site', domain: 'github.com' },
          usedSeconds: 600,
          limitSeconds: 3_600,
        }),
      ],
    });
    renderToday();
    expect(await screen.findByRole('progressbar', { name: 'github.com' })).toBeTruthy();

    await user.click(screen.getByRole('button', { name: 'Previous day' }));

    await waitFor(() => {
      expect(screen.queryByRole('progressbar')).toBeNull();
    });
  });
});

describe('TodayScreen states', () => {
  it('has no accessibility violations while loading, when empty or on error', async () => {
    routeCommands(invokeMock, { get_day_report: () => new Promise(() => undefined) });
    const loading = renderToday();
    await expectAccessible(loading.container, true);
    loading.unmount();

    routeCommands(invokeMock, { get_day_report: ({ date }) => emptyDayReport(date) });
    const empty = renderToday();
    await screen.findByText('Nothing tracked yet');
    await expectAccessible(empty.container, true);
    empty.unmount();

    routeCommands(invokeMock, { get_day_report: () => Promise.reject(new Error('offline')) });
    const failed = renderToday();
    await screen.findByRole('alert');
    await expectAccessible(failed.container, true);
  });

  it('holds a skeleton with real section titles while loading', () => {
    routeCommands(invokeMock, { get_day_report: () => new Promise(() => undefined) });
    renderToday();

    expect(screen.getByRole('status', { name: 'Loading your day' })).toBeTruthy();
    expect(screen.getByRole('heading', { level: 2, name: 'Your day' })).toBeTruthy();
    expect(screen.queryByRole('listbox')).toBeNull();
  });

  it('shows an empty day with a line of direction', async () => {
    routeCommands(invokeMock, { get_day_report: ({ date }) => emptyDayReport(date) });
    renderToday();

    expect(await screen.findByText('Nothing tracked yet')).toBeTruthy();
    expect(
      screen.getByText("Grumpo's watching. Tracking starts as you use your Mac."),
    ).toBeTruthy();
    expect(screen.queryByText('Allow browser access in Settings to see sites.')).toBeNull();
    expect(screen.queryByRole('listbox')).toBeNull();
  });

  it('points to Settings from an empty day when browser access is missing', async () => {
    const user = userEvent.setup();
    routeCommands(invokeMock, {
      get_day_report: ({ date }) => emptyDayReport(date),
      get_permissions: () => MISSING_BROWSER_PERMISSIONS,
    });
    renderToday();

    expect(await screen.findByText('Allow browser access in Settings to see sites.')).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Open Settings' }));

    await waitFor(() => {
      expect(title()).toBe('Settings');
    });
  });

  it('words an empty past day without promising tracking', async () => {
    const user = userEvent.setup();
    routeCommands(invokeMock, {
      get_day_report: ({ date }) =>
        date === '2026-10-12' ? populatedDayReport(date) : emptyDayReport(date),
    });
    renderToday();
    await screen.findByText('1h 40min');

    await user.click(screen.getByRole('button', { name: 'Previous day' }));

    expect(await screen.findByText('Nothing tracked on this day')).toBeTruthy();
    expect(
      screen.queryByText("Grumpo's watching. Tracking starts as you use your Mac."),
    ).toBeNull();
  });

  it('shows an error with a retry that recovers', async () => {
    const user = userEvent.setup();
    let failing = true;
    routeCommands(invokeMock, {
      get_day_report: ({ date }) => {
        if (failing) return Promise.reject(new Error('storage offline'));
        return populatedDayReport(date);
      },
    });
    renderToday();

    expect(await screen.findByRole('alert')).toBeTruthy();
    expect(screen.getByText("Couldn't load your day")).toBeTruthy();

    failing = false;
    await user.click(screen.getByRole('button', { name: 'Try again' }));

    expect(await screen.findByText('1h 40min')).toBeTruthy();
    expect(screen.queryByRole('alert')).toBeNull();
  });
});

describe('TodayScreen first view', () => {
  it('grows the chart from nothing when the data arrives after the screen is already on show', async () => {
    routeCommands(invokeMock);
    renderToday();

    await screen.findByRole('listbox', { name: 'Screen time by hour' });
    const bars = [...document.querySelectorAll<HTMLElement>('[role="option"] > span')];
    expect(bars.some((bar) => bar.style.transform.includes('translateY(100%)'))).toBe(true);
  });
});
