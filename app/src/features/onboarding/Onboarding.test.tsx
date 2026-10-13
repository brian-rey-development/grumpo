import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn<(command: string, ...args: unknown[]) => Promise<unknown>>(),
}));

vi.mock('@/ipc/commands', () => ({ invokeCommand: invokeMock }));

import { INSTALL_COMMAND } from '@/copy/service';
import { OnboardingGate } from '@/features/onboarding/OnboardingGate';
import { SettingsScreen } from '@/features/settings/SettingsScreen';
import { CommandFailure } from '@/ipc/errors';
import type { FocusStatus } from '@/ipc/generated/FocusStatus';
import type { Permissions } from '@/ipc/generated/Permissions';
import { useNavigation } from '@/layout/navigation';
import { type CommandHandlers, connectedStatus, NOT_INSTALLED, routeCommands } from '@/test/ipc';
import { expectAccessible, renderWithProviders } from '@/test/render';

const FRESH_PERMISSIONS: Permissions = {
  accessibility: 'notDetermined',
  browsers: [
    { browser: 'chrome', installed: true, permission: 'notDetermined' },
    { browser: 'firefox', installed: true, permission: 'unsupported' },
  ],
};

interface World {
  completed: boolean;
  status: FocusStatus;
  permissions: Permissions;
}

function setup(overrides: Partial<World> = {}, handlers: CommandHandlers = {}): World {
  const world: World = {
    completed: false,
    status: NOT_INSTALLED,
    permissions: FRESH_PERMISSIONS,
    ...overrides,
  };
  routeCommands(invokeMock, {
    get_onboarding_state: () => ({ completed: world.completed }),
    complete_onboarding: () => {
      world.completed = true;
      return null;
    },
    reset_onboarding: () => {
      world.completed = false;
      return null;
    },
    get_focus_status: () => world.status,
    get_permissions: () => world.permissions,
    ...handlers,
  });
  return world;
}

function CurrentScreen() {
  return <p data-testid="screen">{useNavigation().screen}</p>;
}

function renderOnboarding() {
  return renderWithProviders(
    <OnboardingGate>
      <CurrentScreen />
    </OnboardingGate>,
  );
}

async function heading(name: string): Promise<HTMLElement> {
  return screen.findByRole('heading', { level: 1, name });
}

async function goTo(user: ReturnType<typeof userEvent.setup>, name: string): Promise<void> {
  for (let guard = 0; guard < 6; guard += 1) {
    if (screen.queryByRole('heading', { level: 1, name }) !== null) return;
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(screen.getAllByRole('heading', { level: 1 })).toHaveLength(1);
    });
  }
}

function callsTo(command: string): number {
  return invokeMock.mock.calls.filter(([name]) => name === command).length;
}

function firstOf(items: HTMLElement[]): HTMLElement {
  const [first] = items;
  if (first === undefined) throw new Error('Expected at least one element');
  return first;
}

beforeEach(() => {
  invokeMock.mockReset();
});

describe('flow', () => {
  it('opens on the welcome with one decorative mascot and no Back button available', async () => {
    setup();
    renderOnboarding();

    const title = await heading('Grumpo');
    expect(title.querySelector('img[alt="Grumpo"]')?.getAttribute('alt')).toBe('Grumpo');
    expect(screen.getByText('You get things done.')).toBeTruthy();
    const mascots = document.querySelectorAll('main img[alt=""]');
    expect(mascots).toHaveLength(1);
    expect(screen.getByRole('button', { name: 'Get started' })).toBeTruthy();
    expect(screen.getByRole('progressbar').getAttribute('aria-valuetext')).toBe('Step 1 of 6');
  });

  it('moves forward and back with the footer, one heading at a time', async () => {
    setup();
    const user = userEvent.setup();
    renderOnboarding();
    await heading('Grumpo');

    await user.click(screen.getByRole('button', { name: 'Get started' }));
    await heading('How Grumpo works');
    await waitFor(() => {
      expect(screen.getAllByRole('heading', { level: 1 })).toHaveLength(1);
    });
    expect(screen.getByRole('progressbar').getAttribute('aria-valuetext')).toBe('Step 2 of 6');
    expect(screen.getByText('Tightening is instant.')).toBeTruthy();

    await user.click(screen.getByRole('button', { name: 'Back' }));
    await heading('Grumpo');
    expect(screen.getByRole('progressbar').getAttribute('aria-valuenow')).toBe('1');
  });

  it('moves focus to the heading of each step', async () => {
    setup();
    const user = userEvent.setup();
    renderOnboarding();
    await heading('Grumpo');

    await user.click(screen.getByRole('button', { name: 'Get started' }));

    const next = await heading('How Grumpo works');
    await waitFor(() => {
      expect(document.activeElement).toBe(next);
    });
  });

  it('advances on Return and ignores Escape', async () => {
    setup();
    const user = userEvent.setup();
    renderOnboarding();
    await heading('Grumpo');

    await user.keyboard('{Escape}');
    expect(screen.getByRole('heading', { level: 1, name: 'Grumpo' })).toBeTruthy();
    await user.keyboard('{Enter}');

    await heading('How Grumpo works');
    expect(callsTo('complete_onboarding')).toBe(0);
  });

  it('leaves Return to a focused button instead of also moving on', async () => {
    setup();
    const user = userEvent.setup();
    renderOnboarding();
    await heading('Grumpo');
    await goTo(user, 'Install the Grumpo service');
    await waitFor(() => {
      expect(screen.getAllByRole('heading', { level: 1 })).toHaveLength(1);
    });

    screen.getByRole('button', { name: 'Why Grumpo needs this' }).focus();
    await user.keyboard('{Enter}');

    expect(
      screen.getByRole('heading', { level: 1, name: 'Install the Grumpo service' }),
    ).toBeTruthy();
    expect(
      screen.getByRole('button', { name: 'Why Grumpo needs this' }).getAttribute('aria-expanded'),
    ).toBe('true');
  });

  it('lists what is recorded and what never is', async () => {
    setup();
    const user = userEvent.setup();
    renderOnboarding();
    await heading('Grumpo');

    await goTo(user, 'Private by design');

    expect(screen.getByText(/Everything stays on this Mac/)).toBeTruthy();
    expect(screen.getByText('Keystrokes')).toBeTruthy();
    expect(screen.getByText('Screenshots')).toBeTruthy();
  });
});

describe('install step', () => {
  async function openServiceStep(user: ReturnType<typeof userEvent.setup>) {
    renderOnboarding();
    await heading('Grumpo');
    await goTo(user, 'Install the Grumpo service');
    await waitFor(() => {
      expect(screen.getAllByRole('heading', { level: 1 })).toHaveLength(1);
    });
  }

  it('offers the install and lets the person skip it', async () => {
    setup();
    const user = userEvent.setup();
    await openServiceStep(user);

    expect(screen.getByRole('button', { name: 'Install service' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Do this later' })).toBeTruthy();
    expect(screen.queryByText(INSTALL_COMMAND)).toBeNull();
  });

  it('shows the service as running and turns the skip into Continue once it is', async () => {
    setup({ status: connectedStatus({ active: false }) });
    const user = userEvent.setup();
    await openServiceStep(user);

    expect(await screen.findByText('Running')).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Install service' })).toBeNull();
    expect(screen.getByRole('button', { name: 'Continue' })).toBeTruthy();
  });

  it('installs on one click and then shows it running', async () => {
    const world = setup(
      {},
      {
        install_service: () => {
          world.status = connectedStatus({ active: false });
          return 'installed';
        },
      },
    );
    const user = userEvent.setup();
    await openServiceStep(user);

    await user.click(screen.getByRole('button', { name: 'Install service' }));

    expect(await screen.findByText('Running')).toBeTruthy();
    expect(callsTo('install_service')).toBe(1);
    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'Continue' })).toBeTruthy();
    });
  });

  it('goes quietly back to the button when the password prompt is cancelled', async () => {
    setup({}, { install_service: () => 'cancelled' });
    const user = userEvent.setup();
    await openServiceStep(user);

    await user.click(screen.getByRole('button', { name: 'Install service' }));

    await waitFor(() => {
      expect(callsTo('install_service')).toBe(1);
    });
    await waitFor(() => {
      expect(
        screen.getByRole('button', { name: 'Install service' }).getAttribute('aria-busy'),
      ).toBeNull();
    });
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('explains a failed install and keeps the Terminal command behind a disclosure', async () => {
    setup(
      {},
      {
        install_service: () => {
          throw new CommandFailure('serviceInstallFailed', 'installer exited 1');
        },
      },
    );
    const user = userEvent.setup();
    await openServiceStep(user);

    await user.click(screen.getByRole('button', { name: 'Install service' }));

    const alert = await screen.findByRole('alert');
    expect(alert.textContent).not.toContain('installer exited 1');
    expect(screen.getByRole('button', { name: 'Try again' })).toBeTruthy();
    expect(screen.getByText('Install from Terminal instead')).toBeTruthy();
  });

  it('gives the service the same reasons as the permissions, behind a disclosure', async () => {
    setup();
    const user = userEvent.setup();
    await openServiceStep(user);

    expect(screen.queryByText('What Grumpo never does')).toBeNull();
    await user.click(screen.getByRole('button', { name: 'Why Grumpo needs this' }));

    expect(await screen.findByText('What Grumpo never does')).toBeTruthy();
    expect(screen.getByText(/never sees your password/)).toBeTruthy();
    expect(screen.getByText('Without it')).toBeTruthy();
  });
});

describe('permissions step', () => {
  async function openPermissionsStep(user: ReturnType<typeof userEvent.setup>) {
    renderOnboarding();
    await heading('Grumpo');
    await goTo(user, 'Allow what Grumpo needs');
    await waitFor(() => {
      expect(screen.getAllByRole('heading', { level: 1 })).toHaveLength(1);
    });
  }

  it('shows one row per permission with what it is for, under the same fixed hero box', async () => {
    setup();
    const user = userEvent.setup();
    await openPermissionsStep(user);

    expect(await screen.findByText('Google Chrome')).toBeTruthy();
    expect(screen.getAllByRole('button', { name: 'Allow' })).toHaveLength(2);
    expect(screen.getByText(/which app is in front/)).toBeTruthy();
    expect(screen.getByText(/close a tab on a blocked site/)).toBeTruthy();
    expect(document.querySelectorAll('main img')).toHaveLength(1);
  });

  it('keeps the never and without details behind each card disclosure', async () => {
    setup();
    const user = userEvent.setup();
    await openPermissionsStep(user);
    const toggles = await screen.findAllByRole('button', { name: 'Why Grumpo needs this' });
    expect(toggles).toHaveLength(2);
    expect(screen.queryByText(/never reads what's inside a window/)).toBeNull();

    await user.click(firstOf(toggles));

    expect(await screen.findByText(/never reads what's inside a window/)).toBeTruthy();
    expect(screen.getByText(/aren't quit/)).toBeTruthy();
    expect(toggles[0]?.getAttribute('aria-expanded')).toBe('true');
    expect(toggles[1]?.getAttribute('aria-expanded')).toBe('false');
    expect(screen.queryByText(/Blocked tabs stay open/)).toBeNull();
  });

  it('draws the System Settings path for Accessibility', async () => {
    setup();
    const user = userEvent.setup();
    await openPermissionsStep(user);

    const path = await screen.findByRole('list', { name: 'In System Settings' });
    expect(within(path).getByText('Privacy and Security')).toBeTruthy();
    expect(within(path).getByText('Grumpo')).toBeTruthy();
  });

  it('asks for a permission and picks up the new state, turning the skip into Continue', async () => {
    const world = setup(
      {},
      {
        request_permission: () => {
          world.permissions = {
            accessibility: 'granted',
            browsers: [{ browser: 'chrome', installed: true, permission: 'granted' }],
          };
          return world.permissions;
        },
      },
    );
    const user = userEvent.setup();
    await openPermissionsStep(user);
    expect(screen.getByRole('button', { name: 'Do this later' })).toBeTruthy();

    await user.click(firstOf(await screen.findAllByRole('button', { name: 'Allow' })));

    await waitFor(() => {
      expect(screen.getAllByText('Allowed').length).toBeGreaterThan(0);
    });
    expect(screen.getByRole('button', { name: 'Continue' })).toBeTruthy();
  });

  it('refreshes the status when the window regains focus', async () => {
    const world = setup();
    const user = userEvent.setup();
    await openPermissionsStep(user);
    await screen.findAllByRole('button', { name: 'Allow' });

    world.permissions = {
      accessibility: 'granted',
      browsers: [{ browser: 'chrome', installed: true, permission: 'granted' }],
    };
    window.dispatchEvent(new Event('focus'));

    await waitFor(() => {
      expect(screen.queryByRole('button', { name: 'Allow' })).toBeNull();
    });
  });
});

describe('recap', () => {
  it('lists what is set up and points the rest to Settings without nagging', async () => {
    setup({
      status: { kind: 'connected' } as FocusStatus,
      permissions: {
        accessibility: 'granted',
        browsers: [{ browser: 'chrome', installed: true, permission: 'notDetermined' }],
      },
    });
    const user = userEvent.setup();
    renderOnboarding();
    await heading('Grumpo');
    await goTo(user, "Grumpo's on duty.");

    expect(await screen.findByText('Running')).toBeTruthy();
    expect(screen.getByText('Allowed')).toBeTruthy();
    expect(screen.getByText('Not allowed yet')).toBeTruthy();
    expect(screen.getAllByText('You can do this in Settings.')).toHaveLength(1);
  });
});

describe('finishing', () => {
  it('saves the completion and opens Rules from Add your first rule', async () => {
    setup();
    const user = userEvent.setup();
    renderOnboarding();
    await heading('Grumpo');
    await goTo(user, "Grumpo's on duty.");
    await waitFor(() => {
      expect(screen.getAllByRole('heading', { level: 1 })).toHaveLength(1);
    });

    await user.click(screen.getByRole('button', { name: 'Add your first rule' }));

    await waitFor(() => {
      expect(screen.getByTestId('screen').textContent).toBe('rules');
    });
    expect(callsTo('complete_onboarding')).toBe(1);
  });

  it('opens Today from Go to Today', async () => {
    setup();
    const user = userEvent.setup();
    renderOnboarding();
    await heading('Grumpo');
    await goTo(user, "Grumpo's on duty.");
    await waitFor(() => {
      expect(screen.getAllByRole('heading', { level: 1 })).toHaveLength(1);
    });

    await user.click(screen.getByRole('button', { name: 'Go to Today' }));

    await waitFor(() => {
      expect(screen.getByTestId('screen').textContent).toBe('today');
    });
    expect(callsTo('complete_onboarding')).toBe(1);
  });

  it('stays on the last step and says so when the completion cannot be saved', async () => {
    setup(
      {},
      {
        complete_onboarding: () => {
          throw new CommandFailure('platform', 'disk full');
        },
      },
    );
    const user = userEvent.setup();
    renderOnboarding();
    await heading('Grumpo');
    await goTo(user, "Grumpo's on duty.");
    await waitFor(() => {
      expect(screen.getAllByRole('heading', { level: 1 })).toHaveLength(1);
    });

    await user.click(screen.getByRole('button', { name: 'Add your first rule' }));

    const alert = await screen.findByRole('alert');
    expect(alert.textContent).toContain("Grumpo couldn't save that.");
    expect(screen.queryByTestId('screen')).toBeNull();
  });
});

describe('gate', () => {
  it('never shows the welcome once it was completed, even when the service is missing', async () => {
    setup({ completed: true, status: NOT_INSTALLED });
    renderOnboarding();

    expect(await screen.findByTestId('screen')).toBeTruthy();
    expect(screen.queryByRole('progressbar')).toBeNull();
    expect(screen.queryByRole('dialog')).toBeNull();
  });

  it('opens the app when the state cannot be read, so nobody is locked out', async () => {
    setup(
      {},
      {
        get_onboarding_state: () => {
          throw new CommandFailure('platform', 'unreadable');
        },
      },
    );
    renderOnboarding();

    expect(await screen.findByTestId('screen')).toBeTruthy();
  });

  it('reopens from Settings with Show welcome again', async () => {
    const world = setup({ completed: true, status: connectedStatus({ active: false }) });
    const user = userEvent.setup();
    renderWithProviders(
      <OnboardingGate>
        <SettingsScreen />
      </OnboardingGate>,
      { screen: 'settings' },
    );

    await user.click(await screen.findByRole('button', { name: 'Show welcome again' }));

    await heading('Grumpo');
    expect(callsTo('reset_onboarding')).toBe(1);
    expect(world.completed).toBe(false);
  });

  it('has no accessibility violations on a step', async () => {
    setup();
    const user = userEvent.setup();
    renderOnboarding();
    await heading('Grumpo');
    await goTo(user, 'How Grumpo works');
    await waitFor(() => {
      expect(screen.getAllByRole('heading', { level: 1 })).toHaveLength(1);
    });

    await expectAccessible(document.body, true);
  });
});
