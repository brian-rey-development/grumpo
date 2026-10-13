import { act, fireEvent, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn<(command: string, ...args: unknown[]) => Promise<unknown>>(),
}));

vi.mock('@/ipc/commands', () => ({ invokeCommand: invokeMock }));

import { installRulesCommands } from '@/test/rules';
import { RulesDraftProvider } from '@/features/rules/RulesDraftProvider';
import { RulesScreen } from '@/features/rules/RulesScreen';
import { UnlockProvider } from '@/features/unlock/UnlockProvider';
import { CommandFailure } from '@/ipc/errors';
import type { FocusStatus } from '@/ipc/generated/FocusStatus';
import type { Rules } from '@/ipc/generated/Rules';
import { queryKeys } from '@/ipc/queryKeys';
import { AppShell } from '@/layout/AppShell';
import { connectedStatus, NOT_INSTALLED, SAMPLE_RULES } from '@/test/ipc';
import { expectAccessible, renderWithProviders } from '@/test/render';

const LOOSENING_BODY =
  "These changes weaken focus while it's on. Unlock to save them, or save them when the block ends.";

const IDLE = connectedStatus({ active: false });

const UNLOCKED_FOR_AN_HOUR = {
  kind: 'unlocked',
  until: Math.floor(Date.now() / 1000) + 3_600,
} as const;

interface Daemon {
  status: FocusStatus;
  saved: Rules[];
}

function installDaemon(
  status: FocusStatus = connectedStatus(),
  saveError?: CommandFailure,
): Daemon {
  const daemon: Daemon = { status, saved: [] };
  installRulesCommands(invokeMock, {
    get_focus_status: () => daemon.status,
    save_rules: ({ rules }) => {
      if (saveError) throw saveError;
      daemon.saved.push(rules);
      daemon.status = connectedStatus({ rules });
      return daemon.status;
    },
  });
  return daemon;
}

function renderScreen() {
  return renderWithProviders(
    <RulesDraftProvider>
      <UnlockProvider>
        <AppShell>
          <RulesScreen />
        </AppShell>
      </UnlockProvider>
    </RulesDraftProvider>,
    { screen: 'rules' },
  );
}

async function removeReddit(user: ReturnType<typeof userEvent.setup>) {
  await user.click(await screen.findByRole('button', { name: 'Remove reddit.com' }));
}

beforeEach(() => {
  invokeMock.mockReset();
});

describe('loading', () => {
  it('shows a placeholder while the status is on its way', () => {
    installRulesCommands(invokeMock, { get_focus_status: () => new Promise(() => undefined) });
    renderScreen();

    expect(screen.getByRole('status', { name: 'Loading your rules' })).toBeTruthy();
  });

  it('shows the failure with a way to retry', async () => {
    let healthy = false;
    installRulesCommands(invokeMock, {
      get_focus_status: () => {
        if (!healthy) throw new CommandFailure('internal', 'Socket closed');
        return connectedStatus();
      },
    });
    const user = userEvent.setup();
    renderScreen();

    expect(await screen.findByText('Something went wrong inside Grumpo.')).toBeTruthy();
    expect(screen.queryByText('Socket closed')).toBeNull();
    expect(screen.getByRole('heading', { name: "Rules couldn't be loaded" })).toBeTruthy();
    healthy = true;
    await user.click(screen.getByRole('button', { name: 'Try again' }));

    expect(await screen.findByText('reddit.com')).toBeTruthy();
  });
});

describe('with the service running', () => {
  it('shows the four sections with the saved rules', async () => {
    installDaemon();
    renderScreen();

    expect(await screen.findByText('reddit.com')).toBeTruthy();
    for (const name of ['Blocked sites', 'Blocked apps', 'Focus schedule', 'Daily limits']) {
      expect(screen.getByRole('heading', { name })).toBeTruthy();
    }
    expect(screen.getByText('Slack')).toBeTruthy();
    expect(screen.getByRole('group', { name: 'Focus time 1' })).toBeTruthy();
    expect(await screen.findByText('Used 38min today')).toBeTruthy();
  });

  it('offers Save changes and Discard only once something differs', async () => {
    installDaemon(IDLE);
    const user = userEvent.setup();
    renderScreen();
    await screen.findByText('reddit.com');
    expect(screen.queryByRole('button', { name: 'Save changes' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Discard' })).toBeNull();

    await removeReddit(user);

    expect(await screen.findByRole('button', { name: 'Save changes' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Discard' })).toBeTruthy();
    expect(invokeMock).not.toHaveBeenCalledWith('save_rules', expect.anything());
  });

  it('puts the toolbar actions in the toolbar', async () => {
    installDaemon(IDLE);
    const user = userEvent.setup();
    renderScreen();
    await removeReddit(user);

    const toolbar = screen.getByRole('heading', { level: 1 }).closest('header') ?? document.body;
    expect(within(toolbar).getByRole('button', { name: 'Save changes' })).toBeTruthy();
  });

  it('goes back to the saved rules on Discard', async () => {
    installDaemon(IDLE);
    const user = userEvent.setup();
    renderScreen();
    await removeReddit(user);

    await user.click(await screen.findByRole('button', { name: 'Discard' }));

    expect(screen.getByText('reddit.com')).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Save changes' })).toBeNull();
  });

  it('saves the draft and then has nothing left to save', async () => {
    const daemon = installDaemon(IDLE);
    const user = userEvent.setup();
    renderScreen();
    await removeReddit(user);

    await user.click(await screen.findByRole('button', { name: 'Save changes' }));

    await waitFor(() => {
      expect(screen.queryByRole('button', { name: 'Save changes' })).toBeNull();
    });
    expect(daemon.saved).toHaveLength(1);
    expect(daemon.saved[0]?.blockedSites).toEqual(['youtube.com']);
    expect(screen.queryByText('reddit.com')).toBeNull();
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('saves with Cmd+S', async () => {
    const daemon = installDaemon(IDLE);
    const user = userEvent.setup();
    renderScreen();
    await removeReddit(user);

    fireEvent.keyDown(window, { key: 's', metaKey: true });

    await waitFor(() => {
      expect(daemon.saved).toHaveLength(1);
    });
  });

  it('ignores Cmd+S when there is nothing to save', async () => {
    const daemon = installDaemon(IDLE);
    renderScreen();
    await screen.findByText('reddit.com');

    const notPrevented = fireEvent.keyDown(window, { key: 's', metaKey: true });

    expect(notPrevented).toBe(true);
    expect(daemon.saved).toHaveLength(0);
  });

  it('keeps editing while the saved rules change underneath', async () => {
    installDaemon(IDLE);
    const user = userEvent.setup();
    const { client } = renderScreen();
    await removeReddit(user);

    act(() => {
      client.setQueryData(
        queryKeys.focusStatus,
        connectedStatus({
          active: false,
          rules: { ...SAMPLE_RULES, blockedSites: ['other.com'] },
        }),
      );
    });

    expect(screen.queryByText('reddit.com')).toBeNull();
    expect(screen.getAllByText('youtube.com').length).toBeGreaterThan(0);
    expect(screen.getByRole('button', { name: 'Save changes' })).toBeTruthy();
  });
});

async function addSite(user: ReturnType<typeof userEvent.setup>, site: string) {
  await user.click(await screen.findByRole('button', { name: 'Add site' }));
  await user.type(await screen.findByLabelText('Site'), `${site}{Enter}`);
}

describe('saving while focus is locked', () => {
  it('reads Save changes for a change that only tightens the rules', async () => {
    installDaemon();
    const user = userEvent.setup();
    renderScreen();
    await screen.findByText('reddit.com');

    await addSite(user, 'news.com');

    expect(await screen.findByRole('button', { name: 'Save changes' })).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Unlock to save' })).toBeNull();
  });

  it('reads Unlock to save for a change that loosens locked rules, without trying to save', async () => {
    const daemon = installDaemon();
    const user = userEvent.setup();
    renderScreen();
    await removeReddit(user);

    expect(await screen.findByRole('button', { name: 'Unlock to save' })).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Save changes' })).toBeNull();
    expect(daemon.saved).toHaveLength(0);
  });

  it.each([
    ['there is no block right now', connectedStatus({ active: false })],
    ['focus is unlocked', connectedStatus({ active: true, phase: UNLOCKED_FOR_AN_HOUR })],
  ])('reads Save changes for a loosening change when %s', async (_, status) => {
    installDaemon(status);
    const user = userEvent.setup();
    renderScreen();
    await removeReddit(user);

    expect(await screen.findByRole('button', { name: 'Save changes' })).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Unlock to save' })).toBeNull();
  });

  it('locks raising a daily limit even when there is no block right now', async () => {
    const daemon = installDaemon(IDLE);
    const user = userEvent.setup();
    renderScreen();
    await user.click(
      await screen.findByRole('button', { name: 'Increase the daily limit for youtube.com' }),
    );

    expect(await screen.findByRole('button', { name: 'Unlock to save' })).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Save changes' })).toBeNull();
    expect(daemon.saved).toHaveLength(0);
  });

  it('locks removing a daily limit outside focus time and says what the unlock is for', async () => {
    installDaemon(IDLE);
    const user = userEvent.setup();
    renderScreen();
    await user.click(
      await screen.findByRole('button', { name: 'Remove the limit for youtube.com' }),
    );

    await user.click(await screen.findByRole('button', { name: 'Unlock to save' }));

    const dialog = await screen.findByRole('dialog', { name: 'Unlock focus' });
    expect(within(dialog).getByText('Remove the limit for youtube.com.')).toBeTruthy();
  });

  it('keeps lowering a daily limit and removing a site free outside focus time', async () => {
    installDaemon(IDLE);
    const user = userEvent.setup();
    renderScreen();
    await removeReddit(user);
    await user.click(
      screen.getByRole('button', { name: 'Decrease the daily limit for youtube.com' }),
    );

    expect(await screen.findByRole('button', { name: 'Save changes' })).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Unlock to save' })).toBeNull();
  });

  it('names only the limit in the unlock when a site was removed alongside it outside focus time', async () => {
    installDaemon(IDLE);
    const user = userEvent.setup();
    renderScreen();
    await removeReddit(user);
    await user.click(
      screen.getByRole('button', { name: 'Increase the daily limit for youtube.com' }),
    );

    await user.click(await screen.findByRole('button', { name: 'Unlock to save' }));

    expect(
      await screen.findByText('Raise the limit for youtube.com from 30 minutes to 35 minutes.'),
    ).toBeTruthy();
  });

  it('treats a block caused by a daily limit as locked too', async () => {
    installDaemon(
      connectedStatus({
        active: true,
        reason: 'limit',
        exhausted: [{ kind: 'site', domain: 'youtube.com' }],
      }),
    );
    const user = userEvent.setup();
    renderScreen();
    await removeReddit(user);

    expect(await screen.findByRole('button', { name: 'Unlock to save' })).toBeTruthy();
  });

  it('opens the unlock sheet with a plain description of what loosened', async () => {
    const daemon = installDaemon();
    const user = userEvent.setup();
    renderScreen();
    await removeReddit(user);

    await user.click(await screen.findByRole('button', { name: 'Unlock to save' }));

    const dialog = await screen.findByRole('dialog', { name: 'Unlock focus' });
    expect(await within(dialog).findByText('Change waiting to be saved')).toBeTruthy();
    expect(within(dialog).getByText('Remove reddit.com.')).toBeTruthy();
    expect(daemon.saved).toHaveLength(0);
  });

  it('names several loosened things in the sheet', async () => {
    installDaemon();
    const user = userEvent.setup();
    renderScreen();
    await removeReddit(user);
    await user.click(screen.getByRole('button', { name: 'Remove Slack' }));
    await user.click(
      screen.getByRole('button', { name: 'Increase the daily limit for youtube.com' }),
    );

    await user.click(await screen.findByRole('button', { name: 'Unlock to save' }));

    expect(
      await screen.findByText(
        'Remove reddit.com, stop blocking Slack and raise the limit for youtube.com from 30 minutes to 35 minutes.',
      ),
    ).toBeTruthy();
  });

  it('starts the unlock with Cmd+S instead of a save the service would refuse', async () => {
    const daemon = installDaemon();
    const user = userEvent.setup();
    renderScreen();
    await removeReddit(user);

    fireEvent.keyDown(window, { key: 's', metaKey: true });

    expect(await screen.findByRole('dialog', { name: 'Unlock focus' })).toBeTruthy();
    expect(daemon.saved).toHaveLength(0);
  });

  it('offers Save changes inside the unlocked sheet and saves the draft from there', async () => {
    const daemon = installDaemon();
    const user = userEvent.setup();
    const { client } = renderScreen();
    await removeReddit(user);
    await user.click(await screen.findByRole('button', { name: 'Unlock to save' }));
    const dialog = await screen.findByRole('dialog', { name: 'Unlock focus' });

    act(() => {
      client.setQueryData(
        queryKeys.focusStatus,
        connectedStatus({ active: true, phase: UNLOCKED_FOR_AN_HOUR }),
      );
    });
    const save = await within(dialog).findByRole('button', { name: 'Save changes' });
    expect(within(dialog).queryByText(/go back to Rules/)).toBeNull();
    await user.click(save);

    await waitFor(() => {
      expect(daemon.saved).toHaveLength(1);
    });
    expect(daemon.saved[0]?.blockedSites).toEqual(['youtube.com']);
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
    expect(screen.queryByText('reddit.com')).toBeNull();
  });
});

describe('save failures', () => {
  const REFUSED = new CommandFailure('looseningLocked', 'locked');

  it('keeps the draft and offers to unlock when the service refuses a change the screen thought safe', async () => {
    installDaemon(connectedStatus(), REFUSED);
    const user = userEvent.setup();
    renderScreen();
    await addSite(user, 'news.com');

    await user.click(await screen.findByRole('button', { name: 'Save changes' }));

    const notice = await screen.findByRole('alert');
    expect(notice.textContent).toContain(LOOSENING_BODY);
    expect(within(notice).getByRole('button', { name: 'Start unlock' })).toBeTruthy();
    expect(screen.getAllByText('news.com').length).toBeGreaterThan(0);
    expect(screen.getByRole('button', { name: 'Save changes' })).toBeTruthy();
  });

  it('says limits need an unlock even outside focus time when the service refuses a change', async () => {
    installDaemon(IDLE, REFUSED);
    const user = userEvent.setup();
    renderScreen();
    await addSite(user, 'news.com');

    await user.click(await screen.findByRole('button', { name: 'Save changes' }));

    const notice = await screen.findByRole('alert');
    expect(notice.textContent).toContain('even outside focus time');
    expect(notice.textContent).not.toContain(LOOSENING_BODY);
  });

  it('explains which rule was wrong when the app found a specific fault', async () => {
    installDaemon(
      connectedStatus(),
      new CommandFailure('rulesInvalid', 'schedule window 2 must start before it ends', {
        rulesFault: { kind: 'windowNotForward', number: 2 },
      }),
    );
    const user = userEvent.setup();
    renderScreen();
    await addSite(user, 'news.com');

    await user.click(await screen.findByRole('button', { name: 'Save changes' }));

    const notice = await screen.findByRole('alert');
    expect(notice.textContent).toContain('Focus time 2 has to end after it starts.');
    expect(notice.textContent).not.toContain('schedule window 2');
  });

  it('opens the unlock sheet from the notice with a general description', async () => {
    installDaemon(connectedStatus(), REFUSED);
    const user = userEvent.setup();
    renderScreen();
    await addSite(user, 'news.com');
    await user.click(await screen.findByRole('button', { name: 'Save changes' }));

    await user.click(await screen.findByRole('button', { name: 'Start unlock' }));

    const dialog = await screen.findByRole('dialog', { name: 'Unlock focus' });
    expect(await within(dialog).findByText('These changes weaken your rules.')).toBeTruthy();
  });

  it('hides the notice as soon as the status says focus is unlocked', async () => {
    installDaemon(connectedStatus(), REFUSED);
    const user = userEvent.setup();
    const { client } = renderScreen();
    await addSite(user, 'news.com');
    await user.click(await screen.findByRole('button', { name: 'Save changes' }));
    await screen.findByRole('alert');

    act(() => {
      client.setQueryData(
        queryKeys.focusStatus,
        connectedStatus({ active: true, phase: UNLOCKED_FOR_AN_HOUR }),
      );
    });

    await waitFor(() => {
      expect(screen.queryByRole('alert')).toBeNull();
    });
    expect(screen.getByRole('button', { name: 'Save changes' })).toBeTruthy();
  });

  it('asks the service for its status again after a refusal, in case the screen was out of date', async () => {
    installDaemon(connectedStatus(), REFUSED);
    const user = userEvent.setup();
    renderScreen();
    await addSite(user, 'news.com');
    const before = invokeMock.mock.calls.filter(([name]) => name === 'get_focus_status').length;

    await user.click(await screen.findByRole('button', { name: 'Save changes' }));

    await waitFor(() => {
      const after = invokeMock.mock.calls.filter(([name]) => name === 'get_focus_status').length;
      expect(after).toBeGreaterThan(before);
    });
  });

  it('clears the notice once the draft changes again', async () => {
    installDaemon(connectedStatus(), REFUSED);
    const user = userEvent.setup();
    renderScreen();
    await addSite(user, 'news.com');
    await user.click(await screen.findByRole('button', { name: 'Save changes' }));
    await screen.findByRole('alert');

    await user.click(screen.getByRole('button', { name: 'Discard' }));

    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('explains invalid rules from the code and never shows the service message', async () => {
    installDaemon(
      connectedStatus(),
      new CommandFailure('rulesInvalid', 'schedule window 1 must start before it ends'),
    );
    const user = userEvent.setup();
    renderScreen();
    await addSite(user, 'news.com');

    await user.click(await screen.findByRole('button', { name: 'Save changes' }));

    const notice = await screen.findByRole('alert');
    expect(notice.textContent).toContain('Rules not saved');
    expect(notice.textContent).toContain("Grumpo can't use these rules");
    expect(notice.textContent).not.toContain('schedule window 1 must start before it ends');
    expect(within(notice).queryByRole('button')).toBeNull();
  });

  it('explains any other failure in its own words', async () => {
    installDaemon(
      connectedStatus(),
      new CommandFailure('daemonUnavailable', 'The Grumpo service stopped answering'),
    );
    const user = userEvent.setup();
    renderScreen();
    await addSite(user, 'news.com');

    await user.click(await screen.findByRole('button', { name: 'Save changes' }));

    expect((await screen.findByRole('alert')).textContent).toContain(
      "The Grumpo service isn't running. Open Settings to fix it.",
    );
  });

  it('holds back Save while a focus time cannot work, and says which', async () => {
    installDaemon();
    const user = userEvent.setup();
    renderScreen();
    await screen.findByText('reddit.com');

    await user.selectOptions(
      screen.getByRole('combobox', { name: 'End time for focus time 1' }),
      '08:00',
    );

    expect(
      screen
        .getByRole('button', { name: /^(Save changes|Unlock to save)$/ })
        .hasAttribute('disabled'),
    ).toBe(true);
    expect(screen.getByText('Fix the focus times marked below to save your changes.')).toBeTruthy();
    fireEvent.keyDown(window, { key: 's', metaKey: true });
    expect(invokeMock).not.toHaveBeenCalledWith('save_rules', expect.anything());
  });

  it('leaves Cmd+Shift+S and Cmd+Option+S alone', async () => {
    const daemon = installDaemon();
    const user = userEvent.setup();
    renderScreen();
    await addSite(user, 'news.com');
    await screen.findByRole('button', { name: 'Save changes' });

    const shift = fireEvent.keyDown(window, { key: 'S', metaKey: true, shiftKey: true });
    const option = fireEvent.keyDown(window, { key: 's', metaKey: true, altKey: true });

    expect(shift).toBe(true);
    expect(option).toBe(true);
    expect(daemon.saved).toHaveLength(0);
  });
});

describe('without the service', () => {
  it('shows the last saved rules read only, with a way to Settings', async () => {
    installDaemon();
    const user = userEvent.setup();
    const { client } = renderScreen();
    await screen.findByText('reddit.com');

    act(() => {
      client.setQueryData(queryKeys.focusStatus, NOT_INSTALLED);
    });

    const notice = await screen.findByText(/These are your rules as last saved/);
    expect(notice).toBeTruthy();
    expect(screen.getByText('reddit.com')).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Add site' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Remove reddit.com' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Add app' })).toBeNull();

    await user.click(screen.getByRole('button', { name: 'Open Settings' }));
    expect(await screen.findByRole('heading', { level: 1, name: 'Settings' })).toBeTruthy();
  });

  it('shows the saved rules rather than unsaved edits, and keeps the edits for later', async () => {
    installDaemon(IDLE);
    const user = userEvent.setup();
    const { client } = renderScreen();
    await removeReddit(user);
    expect(screen.queryByText('reddit.com')).toBeNull();

    act(() => {
      client.setQueryData(queryKeys.focusStatus, NOT_INSTALLED);
    });

    expect(await screen.findByText('reddit.com')).toBeTruthy();
    expect(screen.getByText(/Your unsaved changes are kept/)).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Save changes' }).hasAttribute('disabled')).toBe(
      true,
    );
    fireEvent.keyDown(window, { key: 's', metaKey: true });
    expect(invokeMock).not.toHaveBeenCalledWith('save_rules', expect.anything());

    act(() => {
      client.setQueryData(queryKeys.focusStatus, IDLE);
    });

    await waitFor(() => {
      expect(screen.queryByText('reddit.com')).toBeNull();
    });
    expect(screen.queryByText(/Your unsaved changes are kept/)).toBeNull();
    expect(screen.getByRole('button', { name: 'Save changes' }).hasAttribute('disabled')).toBe(
      false,
    );
  });

  it('only promises to keep changes when there are some', async () => {
    installDaemon(IDLE);
    const { client } = renderScreen();
    await screen.findByText('reddit.com');

    act(() => {
      client.setQueryData(queryKeys.focusStatus, NOT_INSTALLED);
    });

    expect(await screen.findByText(/These are your rules as last saved/)).toBeTruthy();
    expect(screen.queryByText(/Your unsaved changes are kept/)).toBeNull();
  });

  it('has nothing to show when the service was never reachable', async () => {
    installDaemon(NOT_INSTALLED);
    const user = userEvent.setup();
    renderScreen();

    expect(await screen.findByRole('heading', { name: "Rules aren't available yet" })).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Open Settings' }));
    expect(await screen.findByRole('heading', { level: 1, name: 'Settings' })).toBeTruthy();
  });
});

describe('accessibility', () => {
  it('has no violations with rules shown', async () => {
    installDaemon();
    const { container } = renderScreen();
    await screen.findByText('Used 38min today');

    await expectAccessible(container, true);
  });

  it('has no violations while the unlock notice is showing', async () => {
    installDaemon(connectedStatus(), new CommandFailure('looseningLocked', 'locked'));
    const user = userEvent.setup();
    const { container } = renderScreen();
    await addSite(user, 'news.com');
    await user.click(await screen.findByRole('button', { name: 'Save changes' }));
    await screen.findByRole('button', { name: 'Start unlock' });

    await expectAccessible(container, true);
  });
});
