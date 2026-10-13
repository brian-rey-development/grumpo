import { act, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';

const { subscribeMock, invokeMock } = vi.hoisted(() => ({
  subscribeMock:
    vi.fn<(event: string, handler: (payload: unknown) => void) => Promise<() => void>>(),
  invokeMock: vi.fn<(command: string, ...args: unknown[]) => Promise<unknown>>(),
}));

vi.mock('@/ipc/events', () => ({ subscribe: subscribeMock }));
vi.mock('@/ipc/commands', () => ({ invokeCommand: invokeMock }));

import { BlockerApp } from '@/blocker/BlockerApp';
import { HEADLINES } from '@/blocker/copy';
import type { BlockScreen } from '@/ipc/generated/BlockScreen';
import { MS_PER_SECOND } from '@/lib/time';
import { expectAccessible } from '@/test/render';

const NOW = new Date(2026, 9, 12, 15, 0, 0);
const NOW_SECONDS = NOW.getTime() / MS_PER_SECOND;
const HOUR = 3_600;

const handlers = new Map<string, (payload: unknown) => void>();

function emit(payload: unknown, event = 'block-screen'): void {
  handlers.get(event)?.(payload);
}
let current: BlockScreen | null = null;

function blockOf(overrides: Partial<BlockScreen> = {}): BlockScreen {
  return {
    id: 1,
    target: { kind: 'site', domain: 'youtube.com' },
    reason: 'schedule',
    until: NOW_SECONDS + 5_400,
    strict: false,
    ...overrides,
  };
}

beforeEach(() => {
  vi.useFakeTimers({ toFake: ['Date', 'setTimeout', 'clearTimeout'] });
  vi.setSystemTime(NOW);
  current = null;
  subscribeMock.mockReset();
  handlers.clear();
  subscribeMock.mockImplementation((event, handler) => {
    handlers.set(event, handler);
    return Promise.resolve(vi.fn());
  });
  invokeMock.mockReset();
  invokeMock.mockImplementation((command) => {
    if (command === 'get_block_screen') return Promise.resolve(current);
    return Promise.resolve(null);
  });
});

afterEach(() => {
  vi.useRealTimers();
});

async function renderBlocker(content: BlockScreen | null) {
  current = content;
  const view = render(<BlockerApp />);
  await act(async () => {
    await Promise.resolve();
    await Promise.resolve();
  });
  return view;
}

function headline(): string {
  return screen.getByRole('heading', { level: 1 }).textContent;
}

it('shows nothing but the empty surface while the backend has no block to show', async () => {
  await renderBlocker(null);

  expect(screen.queryByRole('heading')).toBeNull();
  expect(screen.queryByRole('button')).toBeNull();
});

it('explains a site blocked on a schedule and counts down to the end of focus', async () => {
  await renderBlocker(blockOf());

  expect(HEADLINES).toContain(headline());
  expect(
    screen.getByText("You told Grumpo to block youtube.com. He's taking his job very seriously."),
  ).toBeTruthy();
  expect(screen.getByText('Focus ends in')).toBeTruthy();
  expect(screen.getByText('1:30:00')).toBeTruthy();
  expect(screen.getByText('1 hour 30 minutes')).toBeTruthy();
  expect(screen.queryByText('Strict mode on')).toBeNull();
  expect(screen.getByText("No, Grumpo doesn't take bribes.")).toBeTruthy();
});

it('explains an app that was closed', async () => {
  await renderBlocker(blockOf({ target: { kind: 'app', name: 'Slack' } }));

  expect(screen.getByText('You told Grumpo to keep Slack closed. So he closed it.')).toBeTruthy();
});

it('explains a used up daily limit and counts down to the reset', async () => {
  await renderBlocker(blockOf({ reason: 'limit', until: NOW_SECONDS + 4 * HOUR + 9 }));

  expect(
    screen.getByText("That's all of today's youtube.com. Grumpo is guarding the rest."),
  ).toBeTruthy();
  expect(screen.getByText('Resets in')).toBeTruthy();
  expect(screen.getByText('4:00:09')).toBeTruthy();
});

it('shows no countdown for a block that never ends', async () => {
  await renderBlocker(blockOf({ reason: 'always', until: null }));

  expect(screen.getByText(/You told Grumpo to block youtube.com/)).toBeTruthy();
  expect(screen.queryByText('Focus ends in')).toBeNull();
  expect(screen.queryByText('Resets in')).toBeNull();
  expect(screen.queryByText(/^\d+:\d\d/)).toBeNull();
});

it('marks strict mode with a lock, with or without a countdown', async () => {
  const { unmount } = await renderBlocker(blockOf({ strict: true }));
  const note = screen.getByText('Strict mode on');
  expect(note.querySelector('svg')?.getAttribute('aria-hidden')).toBe('true');
  expect(screen.getByText('Focus ends in')).toBeTruthy();
  unmount();

  await renderBlocker(blockOf({ reason: 'always', until: null, strict: true }));
  expect(screen.getByText('Strict mode on')).toBeTruthy();
  expect(screen.queryByText('Focus ends in')).toBeNull();
});

it('keeps counting every second while the spoken value stays put', async () => {
  await renderBlocker(blockOf({ until: NOW_SECONDS + 65 }));
  expect(screen.getByText('1:05')).toBeTruthy();
  expect(screen.getByText('2 minutes')).toBeTruthy();

  await act(async () => {
    await vi.advanceTimersByTimeAsync(2 * MS_PER_SECOND);
  });

  expect(screen.getByText('1:03')).toBeTruthy();
  expect(screen.getByText('2 minutes')).toBeTruthy();
});

it('hides the digits from screen readers and has no live region that would announce them', async () => {
  const { container } = await renderBlocker(blockOf());

  expect(screen.getByText('1:30:00').getAttribute('aria-hidden')).toBe('true');
  expect(container.querySelector('[aria-live], [role="timer"], [role="status"]')).toBeNull();
});

it('puts focus on Back to work', async () => {
  await renderBlocker(blockOf());

  expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Back to work' }));
});

it('dismisses from the Back to work button', async () => {
  vi.useRealTimers();
  const user = userEvent.setup();
  await renderBlocker(blockOf());

  await user.click(screen.getByRole('button', { name: 'Back to work' }));

  expect(invokeMock).toHaveBeenCalledWith('dismiss_block_screen', { id: 1 });
});

it('dismisses with Escape', async () => {
  vi.useRealTimers();
  const user = userEvent.setup();
  await renderBlocker(blockOf());

  await user.keyboard('{Escape}');

  expect(invokeMock).toHaveBeenCalledWith('dismiss_block_screen', { id: 1 });
});

it('replaces its content and moves to the next headline when the backend sends another block', async () => {
  await renderBlocker(blockOf());
  const before = headline();

  act(() => {
    emit(blockOf({ id: 2, target: { kind: 'app', name: 'Slack' }, reason: 'limit', until: null }));
  });

  expect(
    screen.getByText("That's all of today's Slack. Grumpo is guarding the rest."),
  ).toBeTruthy();
  const next =
    HEADLINES[(HEADLINES.indexOf(before as (typeof HEADLINES)[number]) + 1) % HEADLINES.length];
  expect(headline()).toBe(next);
  expect(screen.queryByText('Focus ends in')).toBeNull();
  expect(screen.getAllByRole('button')).toHaveLength(1);
  expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Back to work' }));
});

it('dismisses the block it shows, not a newer one', async () => {
  vi.useRealTimers();
  const user = userEvent.setup();
  await renderBlocker(blockOf());
  act(() => {
    emit(blockOf({ id: 7 }));
  });

  await user.click(screen.getByRole('button', { name: 'Back to work' }));

  expect(invokeMock).toHaveBeenCalledWith('dismiss_block_screen', { id: 7 });
});

it('goes back to the empty surface when the window hides, so the next show starts clean', async () => {
  await renderBlocker(blockOf());

  act(() => {
    emit(null, 'block-screen-hidden');
  });

  expect(screen.queryByRole('heading')).toBeNull();
  expect(screen.queryByText(/youtube.com/)).toBeNull();

  act(() => {
    emit(blockOf({ id: 2, target: { kind: 'site', domain: 'reddit.com' } }));
  });

  expect(screen.getByText(/block reddit.com/)).toBeTruthy();
});

it('does not let a read that was in flight bring a hidden block back', async () => {
  let resolveRead: (value: BlockScreen) => void = () => undefined;
  invokeMock.mockImplementation((command) =>
    command === 'get_block_screen'
      ? new Promise<BlockScreen>((resolve) => {
          resolveRead = resolve;
        })
      : Promise.resolve(null),
  );
  render(<BlockerApp />);
  await act(async () => {
    await Promise.resolve();
  });

  act(() => {
    emit(null, 'block-screen-hidden');
  });
  await act(async () => {
    resolveRead(blockOf());
    await Promise.resolve();
  });

  expect(screen.queryByRole('heading')).toBeNull();
});

it('does nothing on Escape while no block is showing', async () => {
  vi.useRealTimers();
  const user = userEvent.setup();
  await renderBlocker(null);

  await user.keyboard('{Escape}');

  expect(invokeMock).not.toHaveBeenCalledWith('dismiss_block_screen', expect.anything());
});

it('prefers an event that arrives while the first read is still pending', async () => {
  let resolveRead: (value: BlockScreen) => void = () => undefined;
  invokeMock.mockImplementation((command) =>
    command === 'get_block_screen'
      ? new Promise<BlockScreen>((resolve) => {
          resolveRead = resolve;
        })
      : Promise.resolve(null),
  );
  render(<BlockerApp />);
  await act(async () => {
    await Promise.resolve();
  });

  act(() => {
    emit(blockOf({ id: 2, target: { kind: 'app', name: 'Slack' } }));
  });
  await act(async () => {
    resolveRead(blockOf());
    await Promise.resolve();
  });

  expect(screen.getByText(/keep Slack closed/)).toBeTruthy();
});

it('shows one mascot, which is decorative, and never anything the backend wrote', async () => {
  const { container } = await renderBlocker(blockOf({ strict: true }));

  const images = container.querySelectorAll('img');
  expect(images).toHaveLength(1);
  expect(images[0]?.getAttribute('alt')).toBe('');
  const main = screen.getByRole('main');
  expect(within(main).getAllByRole('button')).toHaveLength(1);
});

it('has no accessibility violations, landmarks included', async () => {
  vi.useRealTimers();
  const { container } = await renderBlocker(blockOf({ strict: true }));

  await expectAccessible(container, true);
});
