import type { QueryClient } from '@tanstack/react-query';
import { act, fireEvent, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn<(command: string, ...args: unknown[]) => Promise<unknown>>(),
}));

vi.mock('@/ipc/commands', () => ({ invokeCommand: invokeMock }));

import { UnlockProvider } from '@/features/unlock/UnlockProvider';
import type { UnlockReason } from '@/features/unlock/types';
import { useUnlock } from '@/features/unlock/unlockContext';
import { CommandFailure } from '@/ipc/errors';
import type { FocusStatus } from '@/ipc/generated/FocusStatus';
import type { UnlockPhaseView } from '@/ipc/generated/UnlockPhaseView';
import { queryKeys } from '@/ipc/queryKeys';
import { formatClock } from '@/lib/format';
import { useNavigation } from '@/layout/navigation';
import { connectedStatus, NOT_INSTALLED, routeCommands, UNLOCK_SENTENCE } from '@/test/ipc';
import { expectAccessible, renderWithProviders } from '@/test/render';

const NOW_MS = Date.UTC(2026, 9, 12, 12, 0, 0);
const NOW_S = NOW_MS / 1000;

const COOLING: UnlockPhaseView = { kind: 'coolingDown', readyAt: NOW_S + 291, minutes: 5 };
const READY: UnlockPhaseView = { kind: 'ready', readyAt: NOW_S - 9, expiresAt: NOW_S + 291 };
const UNLOCKED_UNTIL = NOW_S + 754;
const UNLOCKED: UnlockPhaseView = { kind: 'unlocked', until: UNLOCKED_UNTIL };

const mock = invokeMock;

function Opener({ reason }: { reason?: UnlockReason }) {
  const { open } = useUnlock();
  return (
    <button
      type="button"
      onClick={() => {
        open(reason);
      }}
    >
      Open unlock
    </button>
  );
}

function CurrentScreen() {
  return <p data-testid="screen">{useNavigation().screen}</p>;
}

function renderSheet(reason?: UnlockReason) {
  return renderWithProviders(
    <UnlockProvider>
      <Opener reason={reason ?? { kind: 'manual' }} />
      <CurrentScreen />
    </UnlockProvider>,
  );
}

interface FakeDaemon {
  status: FocusStatus;
  confirm: ReturnType<typeof vi.fn<(args: { phrase: string; minutes: number }) => void>>;
}

function fakeDaemon(initial: FocusStatus): FakeDaemon {
  const daemon: FakeDaemon = { status: initial, confirm: vi.fn() };
  const move = (phase: UnlockPhaseView): FocusStatus => {
    daemon.status = connectedStatus({ phase });
    return daemon.status;
  };
  routeCommands(mock, {
    get_focus_status: () => daemon.status,
    request_unlock: () => move(COOLING),
    cancel_unlock: () => move({ kind: 'locked' }),
    relock: () => move({ kind: 'locked' }),
    confirm_unlock: (args) => {
      daemon.confirm(args);
      return move(UNLOCKED);
    },
  });
  return daemon;
}

async function openSheet(user: ReturnType<typeof userEvent.setup>): Promise<HTMLElement> {
  await user.click(screen.getByRole('button', { name: 'Open unlock' }));
  return await screen.findByRole('dialog');
}

beforeEach(() => {
  mock.mockReset();
  vi.useFakeTimers({ toFake: ['Date'] });
  vi.setSystemTime(NOW_MS);
});

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

describe('useUnlock', () => {
  it('refuses to work outside a provider', () => {
    function Lonely() {
      useUnlock();
      return null;
    }
    const silence = vi.spyOn(console, 'error').mockImplementation(() => undefined);

    expect(() => renderWithProviders(<Lonely />)).toThrow(/UnlockProvider/);
    silence.mockRestore();
  });
});

describe('locked', () => {
  it('explains the wait and offers to start it or keep focusing', async () => {
    fakeDaemon(connectedStatus());
    const user = userEvent.setup();
    renderSheet();

    const dialog = await openSheet(user);

    expect(
      await within(dialog).findByText(/You'll wait 10 minutes before you can unlock/),
    ).toBeTruthy();
    expect(
      within(dialog).getByText(/Each unlock today makes the next wait longer, up to 40 minutes\./),
    ).toBeTruthy();
    expect(within(dialog).getByRole('button', { name: 'Start the wait' })).toBeTruthy();
    expect(screen.getByRole('dialog', { name: 'Unlock focus' })).toBe(dialog);
  });

  it('uses the wait the service reports', async () => {
    fakeDaemon(connectedStatus({ nextCooldownMinutes: 40 }));
    const user = userEvent.setup();
    renderSheet();

    await openSheet(user);

    expect(await screen.findByText(/You'll wait 40 minutes/)).toBeTruthy();
  });

  it('shows the pending rule change when opened because of it', async () => {
    fakeDaemon(connectedStatus());
    const user = userEvent.setup();
    renderSheet({
      kind: 'rulesLocked',
      summary: 'Remove youtube.com and raise the Slack limit.',
      onUnlocked: vi.fn(),
    });

    await openSheet(user);

    expect(await screen.findByText('Change waiting to be saved')).toBeTruthy();
    expect(screen.getByText('Remove youtube.com and raise the Slack limit.')).toBeTruthy();
  });

  it('shows no change summary when opened by hand', async () => {
    fakeDaemon(connectedStatus());
    const user = userEvent.setup();
    renderSheet();

    await openSheet(user);
    await screen.findByText(/You'll wait/);

    expect(screen.queryByText('Change waiting to be saved')).toBeNull();
  });

  it('closes with Keep focusing and puts focus back on the opener', async () => {
    fakeDaemon(connectedStatus());
    const user = userEvent.setup();
    renderSheet();
    await openSheet(user);

    await user.click(await screen.findByRole('button', { name: 'Keep focusing' }));

    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
    expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Open unlock' }));
  });

  it('starts the wait and moves to the countdown', async () => {
    fakeDaemon(connectedStatus());
    const user = userEvent.setup();
    renderSheet();
    await openSheet(user);

    await user.click(await screen.findByRole('button', { name: 'Start the wait' }));

    expect(await screen.findByRole('timer')).toBeTruthy();
    expect(screen.getByRole('dialog', { name: 'Waiting to unlock' })).toBeTruthy();
    expect(mock).toHaveBeenCalledWith('request_unlock');
  });

  it('has no accessibility violations', async () => {
    fakeDaemon(connectedStatus());
    const user = userEvent.setup();
    renderSheet();
    await openSheet(user);
    await screen.findByText(/You'll wait/);

    await expectAccessible(document.body);
  });
});

describe('cooling down', () => {
  it('counts down to the moment it is ready', async () => {
    fakeDaemon(connectedStatus({ phase: COOLING }));
    const user = userEvent.setup();
    renderSheet();
    await openSheet(user);

    const timer = await screen.findByRole('timer');
    expect(timer.textContent).toBe('4:51');
    expect(timer.getAttribute('aria-label')).toContain('4 minutes 51 seconds');
    expect(
      screen.getByText(
        "You can close this. Grumpo will notify you when you can unlock, and you'll have 5 minutes to confirm.",
      ),
    ).toBeTruthy();

    vi.setSystemTime(NOW_MS + 10_000);
    await waitFor(
      () => {
        expect(screen.getByRole('timer').textContent).toBe('4:41');
      },
      { timeout: 2_500 },
    );
  });

  it('draws an amber ring that drains with the wait the request was given', async () => {
    fakeDaemon(connectedStatus({ phase: COOLING }));
    const user = userEvent.setup();
    renderSheet();
    await openSheet(user);
    await screen.findByRole('timer');

    const arc = screen.getByTestId('countdown-arc').querySelectorAll('circle')[1];
    const circumference = 2 * Math.PI * 46;
    const offset = Number(arc?.getAttribute('stroke-dashoffset'));
    expect(offset / circumference).toBeCloseTo(1 - 291 / 300, 2);
  });

  it('can be closed and cancelled', async () => {
    fakeDaemon(connectedStatus({ phase: COOLING }));
    const user = userEvent.setup();
    renderSheet();
    await openSheet(user);

    await user.click(await screen.findByRole('button', { name: 'Cancel unlock' }));

    expect(await screen.findByText(/You'll wait 10 minutes/)).toBeTruthy();
    expect(mock).toHaveBeenCalledWith('cancel_unlock');
  });

  it('closes with the Close button while the wait keeps running', async () => {
    fakeDaemon(connectedStatus({ phase: COOLING }));
    const user = userEvent.setup();
    renderSheet();
    await openSheet(user);

    await user.click(await screen.findByRole('button', { name: 'Close' }));

    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
    expect(mock).not.toHaveBeenCalledWith('cancel_unlock');
  });

  it('asks the service again once the wait is over and shows the next step', async () => {
    let reads = 0;
    routeCommands(mock, {
      get_focus_status: () => {
        reads += 1;
        const phase =
          reads === 1 ? { kind: 'coolingDown' as const, readyAt: NOW_S - 30, minutes: 5 } : READY;
        return connectedStatus({ phase });
      },
    });
    const user = userEvent.setup();
    renderSheet();
    await openSheet(user);

    expect(await screen.findByLabelText('Your sentence')).toBeTruthy();
    expect(screen.getByRole('dialog', { name: 'Unlock ready' })).toBeTruthy();
    expect(reads).toBeGreaterThanOrEqual(2);
  });

  it('has no accessibility violations', async () => {
    fakeDaemon(connectedStatus({ phase: COOLING }));
    const user = userEvent.setup();
    renderSheet();
    await openSheet(user);
    await screen.findByRole('timer');

    await expectAccessible(document.body);
  });
});

describe('ready', () => {
  async function openReady() {
    const daemon = fakeDaemon(connectedStatus({ phase: READY }));
    const user = userEvent.setup();
    renderSheet();
    await openSheet(user);
    const field = await screen.findByLabelText('Your sentence');
    return { daemon, user, field };
  }

  it('shows the sentence, the deadline and a field that already has focus', async () => {
    const { field } = await openReady();

    expect(screen.getByText('Type this sentence to continue')).toBeTruthy();
    expect(screen.getByText(UNLOCK_SENTENCE)).toBeTruthy();
    expect(screen.getByText('Confirm within 4:51')).toBeTruthy();
    expect(document.activeElement).toBe(field);
  });

  it('keeps the unlock button off until the sentence matches exactly', async () => {
    const { user, field } = await openReady();
    const unlock = screen.getByRole('button', { name: 'Unlock for 30 minutes' });
    expect(unlock.hasAttribute('disabled')).toBe(true);

    await user.type(field, UNLOCK_SENTENCE.slice(0, -1));
    expect(unlock.hasAttribute('disabled')).toBe(true);
    expect(field.hasAttribute('data-matches')).toBe(false);

    await user.type(field, '.');
    expect(unlock.hasAttribute('disabled')).toBe(false);
    expect(field.getAttribute('data-matches')).toBe('true');
    expect(screen.getByText('That matches.')).toBeTruthy();
  });

  it('marks how far the typing is right without showing anything new', async () => {
    const { user, field } = await openReady();

    await user.type(field, 'I am x');

    const quote = document.querySelector('blockquote');
    expect([...(quote?.children ?? [])].map((part) => part.textContent).slice(0, 2)).toEqual([
      'I am ',
      'c',
    ]);
    expect(quote?.textContent).toBe(UNLOCK_SENTENCE);
  });

  it('matches like the service does, ignoring extra spaces', async () => {
    const { user, field } = await openReady();

    await user.type(field, `  ${UNLOCK_SENTENCE.replace('I am', 'I   am')}  `);

    expect(
      screen.getByRole('button', { name: 'Unlock for 30 minutes' }).hasAttribute('disabled'),
    ).toBe(false);
  });

  it('does not accept pasted text', async () => {
    const { user, field } = await openReady();

    await user.click(field);
    await user.paste(UNLOCK_SENTENCE);
    const allowed = fireEvent.paste(field, { clipboardData: { getData: () => UNLOCK_SENTENCE } });

    expect(allowed).toBe(false);
    expect((field as HTMLInputElement).value).toBe('');
    expect(field.getAttribute('autocomplete')).toBe('off');
  });

  it('offers 15, 30 and 60 minutes and unlocks for the one chosen', async () => {
    const { daemon, user, field } = await openReady();
    const group = screen.getByRole('radiogroup', { name: 'Unlock for' });
    expect(
      within(group)
        .getAllByRole('radio')
        .map((radio) => radio.getAttribute('aria-label')),
    ).toEqual(['15 minutes', '30 minutes', '1 hour']);
    expect(
      within(group).getByRole('radio', { name: '30 minutes' }).getAttribute('aria-checked'),
    ).toBe('true');

    await user.click(within(group).getByRole('radio', { name: '1 hour' }));
    await user.type(field, UNLOCK_SENTENCE);
    await user.click(screen.getByRole('button', { name: 'Unlock for 1 hour' }));

    expect(daemon.confirm).toHaveBeenCalledExactlyOnceWith({
      phrase: UNLOCK_SENTENCE,
      minutes: 60,
    });
    expect(await screen.findByText(/^Unlocked until/)).toBeTruthy();
  });

  it('submits with Enter once the sentence matches', async () => {
    const { daemon, user, field } = await openReady();

    await user.type(field, `${UNLOCK_SENTENCE}{Enter}`);

    expect(daemon.confirm).toHaveBeenCalledExactlyOnceWith({
      phrase: UNLOCK_SENTENCE,
      minutes: 30,
    });
  });

  it('does not submit with Enter while the sentence is wrong', async () => {
    const { daemon, user, field } = await openReady();

    await user.type(field, 'I am{Enter}');

    expect(daemon.confirm).not.toHaveBeenCalled();
  });

  it('shows the service rejecting the sentence next to the field', async () => {
    fakeDaemon(connectedStatus({ phase: READY }));
    routeCommands(mock, {
      get_focus_status: () => connectedStatus({ phase: READY }),
      confirm_unlock: () => {
        throw new CommandFailure('phraseMismatch', 'phrase does not match');
      },
    });
    const user = userEvent.setup();
    renderSheet();
    await openSheet(user);
    const field = await screen.findByLabelText('Your sentence');

    await user.type(field, `${UNLOCK_SENTENCE}{Enter}`);

    const alert = await screen.findByRole('alert');
    expect(alert.textContent).toContain("That doesn't match the sentence");
    expect(field.getAttribute('aria-invalid')).toBe('true');
    expect(field.getAttribute('aria-describedby')).toBe(alert.id);
  });

  it('explains other failures in plain words', async () => {
    routeCommands(mock, {
      get_focus_status: () => connectedStatus({ phase: READY }),
      confirm_unlock: () => {
        throw new CommandFailure('unlockNotReady', 'not ready');
      },
    });
    const user = userEvent.setup();
    renderSheet();
    await openSheet(user);
    const field = await screen.findByLabelText('Your sentence');

    await user.type(field, `${UNLOCK_SENTENCE}{Enter}`);

    expect((await screen.findByRole('alert')).textContent).toBe(
      "The wait isn't over yet. Grumpo will tell you when it's ready.",
    );
  });

  it('can abandon the request', async () => {
    const { user } = await openReady();

    await user.click(screen.getByRole('button', { name: 'Cancel unlock' }));

    expect(await screen.findByText(/You'll wait 10 minutes/)).toBeTruthy();
  });

  it('has no accessibility violations', async () => {
    await openReady();

    await expectAccessible(document.body);
  });
});

describe('unlocked', () => {
  it('shows when it ends and how long is left', async () => {
    fakeDaemon(connectedStatus({ phase: UNLOCKED }));
    const user = userEvent.setup();
    renderSheet();
    await openSheet(user);

    const until = formatClock(new Date(UNLOCKED_UNTIL * 1000));
    expect(await screen.findByText(`Unlocked until ${until}`)).toBeTruthy();
    expect(screen.getByText('12:34 left')).toBeTruthy();
    expect(screen.getByRole('dialog', { name: 'Focus is unlocked' })).toBeTruthy();
  });

  it('relocks at once', async () => {
    fakeDaemon(connectedStatus({ phase: UNLOCKED }));
    const user = userEvent.setup();
    renderSheet();
    await openSheet(user);

    await user.click(await screen.findByRole('button', { name: 'Relock now' }));

    expect(await screen.findByText(/You'll wait/)).toBeTruthy();
    expect(mock).toHaveBeenCalledWith('relock');
  });

  it('offers to save the waiting change when the unlock was for a rule change', async () => {
    fakeDaemon(connectedStatus({ phase: UNLOCKED }));
    const user = userEvent.setup();
    const onUnlocked = vi.fn();
    renderSheet({ kind: 'rulesLocked', summary: 'Remove youtube.com.', onUnlocked });
    const dialog = await openSheet(user);

    const save = await within(dialog).findByRole('button', { name: 'Save changes' });
    expect(within(dialog).queryByText(/go back to Rules/)).toBeNull();
    expect(within(dialog).getByRole('button', { name: 'Relock now' })).toBeTruthy();
    expect(document.activeElement).toBe(save);

    await user.click(save);

    expect(onUnlocked).toHaveBeenCalledOnce();
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
    expect(mock).not.toHaveBeenCalledWith('relock');
  });

  it('keeps Relock now as the main action when the unlock was opened by hand', async () => {
    fakeDaemon(connectedStatus({ phase: UNLOCKED }));
    const user = userEvent.setup();
    renderSheet();
    const dialog = await openSheet(user);

    await within(dialog).findByRole('button', { name: 'Relock now' });

    expect(within(dialog).queryByRole('button', { name: 'Save changes' })).toBeNull();
    expect(document.activeElement).toBe(within(dialog).getByRole('button', { name: 'Relock now' }));
  });

  it('has no accessibility violations', async () => {
    fakeDaemon(connectedStatus({ phase: UNLOCKED }));
    const user = userEvent.setup();
    renderSheet();
    await openSheet(user);
    await screen.findByText('12:34 left');

    await expectAccessible(document.body);
  });
});

describe('without the focus service', () => {
  it('explains what is missing and links to Settings', async () => {
    fakeDaemon(NOT_INSTALLED);
    const user = userEvent.setup();
    renderSheet();
    await openSheet(user);

    expect(await screen.findByText(/The Grumpo service isn't installed/)).toBeTruthy();
    expect(screen.getByRole('dialog', { name: 'Grumpo service not running' })).toBeTruthy();

    await user.click(screen.getByRole('button', { name: 'Open Settings' }));

    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
    expect(screen.getByTestId('screen').textContent).toBe('settings');
  });

  it.each([
    ['notResponding', /installed but isn't answering/],
    ['serviceOutdated', /older than this app and needs updating/],
    ['appOutdated', /Grumpo needs updating/],
    ['otherUser', /belongs to another user/],
  ] as const)('explains %s', async (reason, text) => {
    fakeDaemon({ kind: 'unavailable', reason, message: 'x' });
    const user = userEvent.setup();
    renderSheet();
    await openSheet(user);

    expect(await screen.findByText(text)).toBeTruthy();
  });

  it('has no accessibility violations', async () => {
    fakeDaemon(NOT_INSTALLED);
    const user = userEvent.setup();
    renderSheet();
    await openSheet(user);
    await screen.findByText(/isn't installed/);

    await expectAccessible(document.body);
  });
});

describe('loading and failing', () => {
  it('shows a placeholder while the first status is on its way', async () => {
    routeCommands(mock, { get_focus_status: () => new Promise(() => undefined) });
    const user = userEvent.setup();
    renderSheet();

    await openSheet(user);

    expect(screen.getByRole('status', { name: 'Checking the Grumpo service' })).toBeTruthy();
  });

  it('shows the failure and recovers on retry', async () => {
    let healthy = false;
    routeCommands(mock, {
      get_focus_status: () => {
        if (!healthy) throw new CommandFailure('internal', 'Socket closed');
        return connectedStatus();
      },
    });
    const user = userEvent.setup();
    renderSheet();
    await openSheet(user);

    expect(await screen.findByText('Something went wrong inside Grumpo.')).toBeTruthy();
    expect(screen.queryByText('Socket closed')).toBeNull();
    healthy = true;
    await user.click(screen.getByRole('button', { name: 'Try again' }));

    expect(await screen.findByText(/You'll wait/)).toBeTruthy();
  });
});

describe('while a command runs', () => {
  it('cannot be dismissed until the command finishes', async () => {
    let finish: (status: FocusStatus) => void = () => undefined;
    routeCommands(mock, {
      get_focus_status: () => connectedStatus(),
      request_unlock: () =>
        new Promise<FocusStatus>((resolve) => {
          finish = resolve;
        }),
    });
    const user = userEvent.setup();
    renderSheet();
    const dialog = await openSheet(user);
    await user.click(await screen.findByRole('button', { name: 'Start the wait' }));

    fireEvent.keyDown(dialog, { key: 'Escape' });
    await user.click(screen.getByTestId('dialog-backdrop'));
    expect(screen.getByRole('dialog')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Keep focusing' }).hasAttribute('disabled')).toBe(
      true,
    );
    expect(screen.getByRole('button', { name: 'Start the wait' }).getAttribute('aria-busy')).toBe(
      'true',
    );

    finish(connectedStatus({ phase: COOLING }));
    await screen.findByRole('timer');
    fireEvent.keyDown(screen.getByRole('dialog'), { key: 'Escape' });
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
  });
});

describe('phase changes while open', () => {
  it('follows the service from waiting to ready and moves focus to the field', async () => {
    fakeDaemon(connectedStatus({ phase: COOLING }));
    const user = userEvent.setup();
    const { client } = renderSheet();
    await openSheet(user);
    await screen.findByRole('timer');

    client.setQueryData(queryKeys.focusStatus, connectedStatus({ phase: READY }));

    const field = await screen.findByLabelText('Your sentence');
    await waitFor(() => {
      expect(document.activeElement).toBe(field);
    });
    expect(screen.queryByRole('timer', { name: /Time left before/ })).toBeNull();
  });

  it('falls back to locked when the unlock window runs out', async () => {
    fakeDaemon(connectedStatus({ phase: READY }));
    const user = userEvent.setup();
    const { client } = renderSheet();
    await openSheet(user);
    await screen.findByLabelText('Your sentence');

    client.setQueryData(queryKeys.focusStatus, connectedStatus());

    expect(await screen.findByText(/You'll wait/)).toBeTruthy();
  });
});

describe('when the time to confirm runs out', () => {
  const TIME_UP = 'The time to confirm is up.';
  const LIVE_REGION = '[role="status"]';

  async function openReadyWithSentence() {
    fakeDaemon(connectedStatus({ phase: READY }));
    const user = userEvent.setup();
    const view = renderSheet();
    await openSheet(user);
    const field = await screen.findByLabelText('Your sentence');
    await user.type(field, UNLOCK_SENTENCE);
    return { user, field, ...view };
  }

  it('turns the confirm button off and says the time is up', async () => {
    const { field } = await openReadyWithSentence();
    const confirm = screen.getByRole('button', { name: 'Unlock for 30 minutes' });
    expect(confirm.hasAttribute('disabled')).toBe(false);

    vi.setSystemTime(NOW_MS + 292_000);

    await waitFor(
      () => {
        expect(screen.getByText(TIME_UP, { selector: LIVE_REGION })).toBeTruthy();
      },
      { timeout: 2_500 },
    );
    expect(screen.getByText(TIME_UP, { selector: `:not(${LIVE_REGION})` })).toBeTruthy();
    expect(
      screen.getByRole('button', { name: 'Unlock for 30 minutes' }).hasAttribute('disabled'),
    ).toBe(true);
    expect(screen.queryByText(/Confirm within/)).toBeNull();
    fireEvent.submit(field.closest('form') ?? field);
    expect(mock).not.toHaveBeenCalledWith('confirm_unlock', expect.anything());
  });

  it('does not submit with Enter once the time is up', async () => {
    const { user } = await openReadyWithSentence();
    vi.setSystemTime(NOW_MS + 292_000);
    await screen.findByText(TIME_UP, { selector: LIVE_REGION }, { timeout: 2_500 });

    await user.keyboard('{Enter}');

    expect(mock).not.toHaveBeenCalledWith('confirm_unlock', expect.anything());
  });
});

describe('a confirmation that ran out', () => {
  const RAN_OUT = "Your 5 minutes to confirm ran out. Start again when you're ready.";

  it('is explained in the locked view when it lapsed while the sheet was open', async () => {
    fakeDaemon(connectedStatus({ phase: READY }));
    const user = userEvent.setup();
    const { client } = renderSheet();
    await openSheet(user);
    await screen.findByLabelText('Your sentence');

    vi.setSystemTime(NOW_MS + 295_000);
    act(() => {
      client.setQueryData(queryKeys.focusStatus, connectedStatus());
    });

    expect(await screen.findByText(RAN_OUT)).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Start the wait' })).toBeTruthy();
  });

  it('is still explained when it lapsed while the sheet was closed', async () => {
    fakeDaemon(connectedStatus({ phase: READY }));
    const user = userEvent.setup();
    const { client } = renderSheet();
    await waitFor(() => {
      expect(client.getQueryData(queryKeys.focusStatus)).toBeTruthy();
    });

    vi.setSystemTime(NOW_MS + 295_000);
    act(() => {
      client.setQueryData(queryKeys.focusStatus, connectedStatus());
    });
    await openSheet(user);

    expect(await screen.findByText(RAN_OUT)).toBeTruthy();
  });

  it('is not claimed when the person cancelled in time', async () => {
    fakeDaemon(connectedStatus({ phase: READY }));
    const user = userEvent.setup();
    renderSheet();
    await openSheet(user);

    await user.click(await screen.findByRole('button', { name: 'Cancel unlock' }));

    expect(await screen.findByText(/You'll wait/)).toBeTruthy();
    expect(screen.queryByText(RAN_OUT)).toBeNull();
  });

  it('goes away once a new wait starts', async () => {
    fakeDaemon(connectedStatus({ phase: READY }));
    const user = userEvent.setup();
    const { client } = renderSheet();
    await openSheet(user);
    await screen.findByLabelText('Your sentence');
    vi.setSystemTime(NOW_MS + 295_000);
    act(() => {
      client.setQueryData(queryKeys.focusStatus, connectedStatus());
    });
    await screen.findByText(RAN_OUT);

    await user.click(screen.getByRole('button', { name: 'Start the wait' }));
    await screen.findByRole('timer');
    await user.click(screen.getByRole('button', { name: 'Cancel unlock' }));

    expect(await screen.findByText(/You'll wait/)).toBeTruthy();
    expect(screen.queryByText(RAN_OUT)).toBeNull();
  });
});

describe('a failure that outlives a change of phase', () => {
  it('still shows when the phase moved right after the click', async () => {
    const view: { client?: QueryClient } = {};
    routeCommands(mock, {
      get_focus_status: () => connectedStatus({ phase: READY }),
      confirm_unlock: () => {
        view.client?.setQueryData(queryKeys.focusStatus, connectedStatus());
        throw new CommandFailure('unlockNotRequested', 'no unlock waiting');
      },
    });
    const user = userEvent.setup();
    const rendered = renderSheet();
    view.client = rendered.client;
    await openSheet(user);
    const field = await screen.findByLabelText('Your sentence');

    await user.type(field, `${UNLOCK_SENTENCE}{Enter}`);

    expect(await screen.findByText('No unlock is waiting. Start the wait again.')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Start the wait' })).toBeTruthy();
  });

  it('goes away when the person acts again', async () => {
    const view: { client?: QueryClient } = {};
    routeCommands(mock, {
      get_focus_status: () => connectedStatus({ phase: READY }),
      confirm_unlock: () => {
        view.client?.setQueryData(queryKeys.focusStatus, connectedStatus());
        throw new CommandFailure('unlockNotRequested', 'no unlock waiting');
      },
      request_unlock: () => connectedStatus({ phase: COOLING }),
    });
    const user = userEvent.setup();
    const rendered = renderSheet();
    view.client = rendered.client;
    await openSheet(user);
    await user.type(await screen.findByLabelText('Your sentence'), `${UNLOCK_SENTENCE}{Enter}`);
    await screen.findByText('No unlock is waiting. Start the wait again.');

    await user.click(screen.getByRole('button', { name: 'Start the wait' }));

    await screen.findByRole('timer');
    expect(screen.queryByText('No unlock is waiting. Start the wait again.')).toBeNull();
  });

  it('goes away when the sheet is closed and opened again', async () => {
    const view: { client?: QueryClient } = {};
    routeCommands(mock, {
      get_focus_status: () => connectedStatus({ phase: READY }),
      confirm_unlock: () => {
        view.client?.setQueryData(queryKeys.focusStatus, connectedStatus());
        throw new CommandFailure('unlockNotRequested', 'no unlock waiting');
      },
    });
    const user = userEvent.setup();
    const rendered = renderSheet();
    view.client = rendered.client;
    await openSheet(user);
    await user.type(await screen.findByLabelText('Your sentence'), `${UNLOCK_SENTENCE}{Enter}`);
    await screen.findByText('No unlock is waiting. Start the wait again.');

    await user.click(screen.getByRole('button', { name: 'Keep focusing' }));
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
    await openSheet(user);

    await screen.findByText(/You'll wait/);
    expect(screen.queryByText('No unlock is waiting. Start the wait again.')).toBeNull();
  });
});
