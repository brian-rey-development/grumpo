import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn<(command: string, ...args: unknown[]) => Promise<unknown>>(),
}));

vi.mock('@/ipc/commands', () => ({ invokeCommand: invokeMock }));

import { SettingsScreen } from '@/features/settings/SettingsScreen';
import { INSTALL_COMMAND } from '@/copy/service';
import { CommandFailure } from '@/ipc/errors';
import type { CommandErrorCode } from '@/ipc/generated/CommandErrorCode';
import type { FocusStatus } from '@/ipc/generated/FocusStatus';
import type { Permissions } from '@/ipc/generated/Permissions';
import type { Settings } from '@/ipc/generated/Settings';
import {
  ALLOWED_PERMISSIONS,
  type CommandHandlers,
  connectedStatus,
  DEFAULT_PERMISSIONS,
  DEFAULT_SETTINGS,
  EMPTY_RULES,
  NOT_INSTALLED,
  routeCommands,
} from '@/test/ipc';
import { expectAccessible, renderWithProviders } from '@/test/render';

const EXPORT_PATH = '/Users/someone/Downloads/Focus export 2026-10-12 1530.json';

interface World {
  status: FocusStatus;
  settings: Settings;
  permissions: Permissions;
}

function install(overrides: Partial<World> = {}, handlers: CommandHandlers = {}): World {
  const world: World = {
    status: connectedStatus(),
    settings: DEFAULT_SETTINGS,
    permissions: DEFAULT_PERMISSIONS,
    ...overrides,
  };
  routeCommands(invokeMock, {
    get_focus_status: () => world.status,
    get_settings: () => world.settings,
    update_settings: ({ settings }) => {
      world.settings = settings;
      return settings;
    },
    get_permissions: () => world.permissions,
    request_permission: () => {
      world.permissions = {
        ...world.permissions,
        browsers: world.permissions.browsers.map((entry) => ({ ...entry, permission: 'granted' })),
      };
      return world.permissions;
    },
    open_permission_settings: () => null,
    export_data: () => EXPORT_PATH,
    delete_all_data: () => null,
    ...handlers,
  });
  return world;
}

function renderSettings() {
  return renderWithProviders(<SettingsScreen />, { screen: 'settings' });
}

function section(name: string): HTMLElement {
  return screen.getByRole('group', { name });
}

beforeEach(() => {
  invokeMock.mockReset();
});

describe('focus service', () => {
  const OUTDATED: FocusStatus = {
    kind: 'unavailable',
    reason: 'serviceOutdated',
    message: 'protocol 2 vs 3',
  };
  const NOT_RESPONDING: FocusStatus = {
    kind: 'unavailable',
    reason: 'notResponding',
    message: 'timeout',
  };

  function failInstall(code: CommandErrorCode): () => never {
    return () => {
      throw new CommandFailure(code, 'installer exited 1 at /tmp/focus-install.log');
    };
  }

  function installCalls(): number {
    return invokeMock.mock.calls.filter(([command]) => command === 'install_service').length;
  }

  it('says the service is running', async () => {
    install();
    renderSettings();

    expect(await screen.findByText('Running')).toBeTruthy();
    expect(
      screen.getByText('The Grumpo service and this app are on the same version.'),
    ).toBeTruthy();
    expect(screen.queryByRole('button', { name: /service$/ })).toBeNull();
  });

  it('offers to install a missing service, with the password prompt explained and no command', async () => {
    install({ status: NOT_INSTALLED });
    renderSettings();

    expect(await screen.findByText(/The Grumpo service isn't installed/)).toBeTruthy();
    expect(
      screen.getByText('macOS will ask for your Mac password once, to install the service.'),
    ).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Install service' })).toBeTruthy();
    expect(screen.queryByText(INSTALL_COMMAND)).toBeNull();
    expect(screen.queryByRole('button', { name: 'Check again' })).toBeNull();
  });

  it('does not repeat the backend message next to the explanation', async () => {
    install({ status: NOT_INSTALLED });
    renderSettings();

    await screen.findByText(/The Grumpo service isn't installed/);

    expect(screen.queryByText(/No socket at/)).toBeNull();
  });

  it('installs on one click, shows progress, then switches to running', async () => {
    let finish: () => void = () => undefined;
    const world = install(
      { status: NOT_INSTALLED },
      {
        install_service: () =>
          new Promise<'installed'>((resolve) => {
            finish = () => {
              world.status = connectedStatus();
              resolve('installed');
            };
          }),
      },
    );
    const user = userEvent.setup();
    renderSettings();

    await user.click(await screen.findByRole('button', { name: 'Install service' }));

    const busy = screen.getByRole('button', { name: 'Install service' });
    expect(busy.getAttribute('aria-busy')).toBe('true');
    expect(screen.getByRole('status').textContent).toContain('Installing the Grumpo service');
    expect(invokeMock).toHaveBeenCalledWith('install_service');
    finish();
    expect(await screen.findByText('Running')).toBeTruthy();
    expect(installCalls()).toBe(1);
  });

  it('offers to update an outdated service, named for what it does', async () => {
    install({ status: OUTDATED });
    renderSettings();

    expect(await screen.findByText(/older than this app and needs updating/)).toBeTruthy();
    expect(
      screen.getByText('macOS will ask for your Mac password once, to update the service.'),
    ).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Update service' })).toBeTruthy();
    expect(screen.queryByText(/protocol 2 vs 3/)).toBeNull();
  });

  it('goes quietly back to the button when the password prompt is cancelled', async () => {
    install({ status: NOT_INSTALLED }, { install_service: () => 'cancelled' });
    const user = userEvent.setup();
    renderSettings();

    await user.click(await screen.findByRole('button', { name: 'Install service' }));

    await waitFor(() => {
      expect(installCalls()).toBe(1);
    });
    const button = await screen.findByRole('button', { name: 'Install service' });
    await waitFor(() => {
      expect(button.getAttribute('aria-busy')).toBeNull();
    });
    expect(screen.queryByRole('alert')).toBeNull();
    expect(screen.queryByText(INSTALL_COMMAND)).toBeNull();
    expect(screen.getByRole('status').textContent).toBe('');
  });

  it('explains a failed install in words, offers Try again and keeps Terminal behind a disclosure', async () => {
    install({ status: NOT_INSTALLED }, { install_service: failInstall('serviceInstallFailed') });
    const user = userEvent.setup();
    renderSettings();

    await user.click(await screen.findByRole('button', { name: 'Install service' }));

    const alert = await screen.findByRole('alert');
    expect(alert.textContent).toBe(
      "The Grumpo service couldn't be installed. Try again, or install it from Terminal instead.",
    );
    expect(screen.queryByText(/installer exited/)).toBeNull();
    expect(screen.getByRole('button', { name: 'Try again' })).toBeTruthy();
    const disclosure = screen.getByText('Install from Terminal instead').closest('details');
    expect(disclosure?.open).toBe(false);
  });

  it('shows the Terminal command inside the disclosure, with a way to copy it and check again', async () => {
    install({ status: NOT_INSTALLED }, { install_service: failInstall('serviceInstallFailed') });
    const user = userEvent.setup();
    renderSettings();
    await user.click(await screen.findByRole('button', { name: 'Install service' }));

    await user.click(await screen.findByText('Install from Terminal instead'));

    expect(screen.getByLabelText('Install command').textContent).toBe(INSTALL_COMMAND);
    expect(
      screen.getByText(/Run this command in Terminal from the Grumpo project folder/),
    ).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Copy' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Check again' })).toBeTruthy();
  });

  it('tries again after a failure and recovers when it works', async () => {
    let attempts = 0;
    const world = install(
      { status: NOT_INSTALLED },
      {
        install_service: () => {
          attempts += 1;
          if (attempts === 1) throw new CommandFailure('serviceInstallFailed', 'exit 1');
          world.status = connectedStatus();
          return 'installed';
        },
      },
    );
    const user = userEvent.setup();
    renderSettings();
    await user.click(await screen.findByRole('button', { name: 'Install service' }));

    await user.click(await screen.findByRole('button', { name: 'Try again' }));

    expect(await screen.findByText('Running')).toBeTruthy();
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('shows the Terminal command outright in a build that cannot install', async () => {
    install(
      { status: NOT_INSTALLED },
      { install_service: failInstall('serviceInstallUnavailable') },
    );
    const user = userEvent.setup();
    renderSettings();

    await user.click(await screen.findByRole('button', { name: 'Install service' }));

    expect(await screen.findByLabelText('Install command')).toBeTruthy();
    expect(screen.getByText(/can't install the service by itself/)).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Install service' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Try again' })).toBeNull();
    expect(screen.queryByRole('alert')).toBeNull();
    expect(screen.queryByText('Install from Terminal instead')).toBeNull();
  });

  it('only says Grumpo needs updating when the service is newer than the app', async () => {
    install({ status: { kind: 'unavailable', reason: 'appOutdated', message: 'protocol 4' } });
    renderSettings();

    expect(await screen.findByText(/Grumpo needs updating/)).toBeTruthy();
    expect(screen.queryByLabelText(/command/)).toBeNull();
    expect(screen.queryByRole('button', { name: /service$/ })).toBeNull();
  });

  it('says the service belongs to another user, with nothing to run', async () => {
    install({ status: { kind: 'unavailable', reason: 'otherUser', message: 'uid 502' } });
    renderSettings();

    expect(
      await screen.findByText('The Grumpo service on this Mac belongs to another user.'),
    ).toBeTruthy();
    expect(screen.queryByLabelText(/command/)).toBeNull();
    expect(screen.queryByRole('button', { name: /service$/ })).toBeNull();
  });

  it('explains a service that does not answer without a command, since it restarts on its own', async () => {
    install({ status: NOT_RESPONDING });
    renderSettings();

    expect(await screen.findByText(/installed but isn't answering/)).toBeTruthy();
    expect(screen.queryByLabelText(/command/)).toBeNull();
    expect(screen.queryByRole('button', { name: /service$/ })).toBeNull();
    expect(screen.getByRole('button', { name: 'Check again' })).toBeTruthy();
  });

  it('copies the command', async () => {
    install(
      { status: NOT_INSTALLED },
      { install_service: failInstall('serviceInstallUnavailable') },
    );
    const user = userEvent.setup();
    renderSettings();
    await user.click(await screen.findByRole('button', { name: 'Install service' }));

    await user.click(await screen.findByRole('button', { name: 'Copy' }));

    expect(await navigator.clipboard.readText()).toBe(INSTALL_COMMAND);
    expect(await screen.findByRole('button', { name: 'Copied' })).toBeTruthy();
  });

  it('says when the command could not be copied', async () => {
    install(
      { status: NOT_INSTALLED },
      { install_service: failInstall('serviceInstallUnavailable') },
    );
    const user = userEvent.setup();
    renderSettings();
    await user.click(await screen.findByRole('button', { name: 'Install service' }));
    vi.spyOn(navigator.clipboard, 'writeText').mockRejectedValue(new Error('denied'));

    await user.click(await screen.findByRole('button', { name: 'Copy' }));

    expect((await screen.findByRole('alert')).textContent).toContain("couldn't be copied");
  });

  it('checks again and reports the service is still down', async () => {
    install({ status: NOT_RESPONDING });
    const user = userEvent.setup();
    renderSettings();
    await screen.findByText(/isn't answering/);
    const before = invokeMock.mock.calls.filter(
      ([command]) => command === 'get_focus_status',
    ).length;

    await user.click(screen.getByRole('button', { name: 'Check again' }));

    expect(await screen.findByText("The Grumpo service still isn't running.")).toBeTruthy();
    const after = invokeMock.mock.calls.filter(
      ([command]) => command === 'get_focus_status',
    ).length;
    expect(after).toBe(before + 1);
  });

  it('switches to the running state once the service answers', async () => {
    const world = install({ status: NOT_RESPONDING });
    const user = userEvent.setup();
    renderSettings();
    await screen.findByText(/isn't answering/);

    world.status = connectedStatus();
    await user.click(screen.getByRole('button', { name: 'Check again' }));

    expect(await screen.findByText('Running')).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Check again' })).toBeNull();
  });
});

describe('permissions', () => {
  it('explains why in one line and lists only installed browsers', async () => {
    install();
    renderSettings();

    const browser = await screen.findByRole('list', { name: 'Permissions' });
    expect(screen.getByText(/so it can see which app and site you're in/)).toBeTruthy();
    expect(screen.getByText(/Nothing leaves your Mac/)).toBeTruthy();
    expect(within(browser).getAllByRole('listitem')).toHaveLength(3);
    expect(screen.queryByText('Brave')).toBeNull();
  });

  it('shows Firefox as covered by Accessibility, with nothing of its own to allow', async () => {
    install({
      permissions: ALLOWED_PERMISSIONS,
    });
    renderSettings();

    const list = await screen.findByRole('list', { name: 'Permissions' });
    const firefox = rowOf('Firefox');

    expect(within(list).getAllByRole('listitem')).toHaveLength(3);
    expect(firefox.textContent).toContain('Covered by Accessibility');
    expect(firefox.querySelector('button')).toBeNull();
    expect(screen.queryByText('Not available on this Mac')).toBeNull();
  });

  it('asks for Accessibility on the Firefox row while Accessibility is not allowed', async () => {
    install({
      permissions: {
        accessibility: 'denied',
        browsers: [{ browser: 'firefox', installed: true, permission: 'unsupported' }],
      },
    });
    renderSettings();

    const list = await screen.findByRole('list', { name: 'Permissions' });

    expect(within(list).getByText('Needs Accessibility')).toBeTruthy();
  });

  it('shows each state in words with the matching action', async () => {
    install();
    renderSettings();

    const rows = await screen.findAllByRole('listitem');
    const text = (name: string) =>
      rows.find((row) => row.textContent.startsWith(name))?.textContent ?? '';
    expect(text('Accessibility')).toContain('Allowed');
    expect(text('Google Chrome')).toContain('Not asked yet');
    expect(text('Safari')).toContain('Not allowed');
    expect(text('Safari')).toContain('Turn Grumpo on under Privacy and Security, then Automation.');
    expect(screen.getAllByRole('button', { name: 'Allow' })).toHaveLength(1);
    expect(screen.getAllByRole('button', { name: 'Open System Settings' })).toHaveLength(1);
  });

  it('asks for permission and shows the new state', async () => {
    install();
    const user = userEvent.setup();
    renderSettings();

    await user.click(await screen.findByRole('button', { name: 'Allow' }));

    expect(invokeMock).toHaveBeenCalledWith('request_permission', {
      permission: { kind: 'automation', browser: 'chrome' },
    });
    await waitFor(() => {
      expect(screen.queryByRole('button', { name: 'Allow' })).toBeNull();
    });
  });

  it('opens System Settings for a denied permission', async () => {
    install();
    const user = userEvent.setup();
    renderSettings();

    await user.click(await screen.findByRole('button', { name: 'Open System Settings' }));

    expect(invokeMock).toHaveBeenCalledWith('open_permission_settings', {
      permission: { kind: 'automation', browser: 'safari' },
    });
  });

  it('draws the System Settings path when Accessibility is denied', async () => {
    install({ permissions: { accessibility: 'denied', browsers: [] } });
    renderSettings();

    const path = await screen.findByRole('list', { name: 'In System Settings' });

    expect(within(path).getByText('Privacy and Security')).toBeTruthy();
    expect(within(path).getByText('Accessibility')).toBeTruthy();
  });

  it('offers both Allow and Open System Settings when Accessibility is denied', async () => {
    install({ permissions: { accessibility: 'denied', browsers: [] } });
    const user = userEvent.setup();
    renderSettings();
    await screen.findByRole('list', { name: 'In System Settings' });
    const row = within(rowOf('Accessibility'));

    await user.click(row.getByRole('button', { name: 'Allow' }));
    expect(invokeMock).toHaveBeenCalledWith('request_permission', {
      permission: { kind: 'accessibility' },
    });

    await user.click(row.getByRole('button', { name: 'Open System Settings' }));
    expect(invokeMock).toHaveBeenCalledWith('open_permission_settings', {
      permission: { kind: 'accessibility' },
    });
  });

  it('does not offer Allow for a denied browser, which can only be changed in System Settings', async () => {
    install();
    renderSettings();
    await screen.findByText('Not asked yet');

    const safari = within(rowOf('Safari'));

    expect(safari.queryByRole('button', { name: 'Allow' })).toBeNull();
    expect(safari.getByRole('button', { name: 'Open System Settings' })).toBeTruthy();
  });

  it('asks to open a browser that is not running, with no button to press', async () => {
    install({
      permissions: {
        accessibility: 'granted',
        browsers: [
          { browser: 'chrome', installed: true, permission: 'notRunning' },
          { browser: 'safari', installed: true, permission: 'granted' },
        ],
      },
    });
    renderSettings();

    const chrome = await screen.findByText('Open Google Chrome to check access.');

    expect(
      within(rowOf('Google Chrome')).queryByRole('button', {
        name: /^(Allow|Open System Settings)$/,
      }),
    ).toBeNull();
    expect(chrome).toBeTruthy();
    expect(within(rowOf('Safari')).getByText('Allowed')).toBeTruthy();
  });

  it('shows pending on the row that was asked and leaves the others usable', async () => {
    let finish: () => void = () => undefined;
    install(
      {
        permissions: {
          accessibility: 'notDetermined',
          browsers: [{ browser: 'chrome', installed: true, permission: 'notDetermined' }],
        },
      },
      {
        request_permission: () =>
          new Promise<Permissions>((resolve) => {
            finish = () => {
              resolve({ accessibility: 'notDetermined', browsers: [] });
            };
          }),
      },
    );
    const user = userEvent.setup();
    renderSettings();
    await screen.findByText('Google Chrome');

    await user.click(within(rowOf('Google Chrome')).getByRole('button', { name: 'Allow' }));

    const chrome = within(rowOf('Google Chrome')).getByRole('button', { name: 'Allow' });
    const accessibility = within(rowOf('Accessibility')).getByRole('button', { name: 'Allow' });
    expect(chrome.getAttribute('aria-busy')).toBe('true');
    expect(accessibility.getAttribute('aria-busy')).toBeNull();
    expect(accessibility.hasAttribute('disabled')).toBe(false);
    finish();
  });

  it('shows a failure on the row it belongs to', async () => {
    install(
      {
        permissions: {
          accessibility: 'notDetermined',
          browsers: [{ browser: 'chrome', installed: true, permission: 'notDetermined' }],
        },
      },
      {
        request_permission: ({ permission }) => {
          if (permission.kind === 'accessibility') {
            throw new CommandFailure('platform', "macOS didn't allow that.");
          }
          return { accessibility: 'notDetermined', browsers: [] };
        },
      },
    );
    const user = userEvent.setup();
    renderSettings();
    await screen.findByText('Google Chrome');

    await user.click(within(rowOf('Accessibility')).getByRole('button', { name: 'Allow' }));

    const alert = await within(rowOf('Accessibility')).findByRole('alert');
    expect(alert.textContent).toContain(
      "Accessibility couldn't be changed. macOS didn't allow that.",
    );
    expect(within(rowOf('Google Chrome')).queryByRole('alert')).toBeNull();
  });

  it('shows unsupported permissions without any button', async () => {
    install({ permissions: { accessibility: 'unsupported', browsers: [] } });
    renderSettings();

    expect(await screen.findByText('Not available on this Mac')).toBeTruthy();
    expect(
      within(screen.getByRole('list', { name: 'Permissions' })).queryByRole('button', {
        name: /^(Allow|Open System Settings)$/,
      }),
    ).toBeNull();
  });

  it('shows what went wrong when asking fails', async () => {
    install(
      {},
      {
        request_permission: () => {
          throw new CommandFailure('platform', "macOS didn't allow that.");
        },
      },
    );
    const user = userEvent.setup();
    renderSettings();

    await user.click(await screen.findByRole('button', { name: 'Allow' }));

    const alert = await screen.findByRole('alert');
    expect(alert.textContent).toContain(
      "Google Chrome couldn't be changed. macOS didn't allow that.",
    );
    expect(alert.textContent).toContain("macOS didn't allow that.");
  });

  it('looks again when the window gets focus back from System Settings', async () => {
    install();
    renderSettings();
    await screen.findByText('Not asked yet');
    const reads = () =>
      invokeMock.mock.calls.filter(([command]) => command === 'get_permissions').length;
    const before = reads();

    fireEvent.focus(window);

    await waitFor(() => {
      expect(reads()).toBe(before + 1);
    });
  });

  it('shows a failure to read permissions with a retry', async () => {
    let healthy = false;
    install(
      {},
      {
        get_permissions: () => {
          if (!healthy)
            throw new CommandFailure('platform', 'No access to the permission database');
          return DEFAULT_PERMISSIONS;
        },
      },
    );
    const user = userEvent.setup();
    renderSettings();

    expect(await screen.findByText("macOS didn't allow that.")).toBeTruthy();
    expect(screen.queryByText('No access to the permission database')).toBeNull();
    healthy = true;
    await user.click(within(section('Permissions')).getByRole('button', { name: 'Try again' }));

    expect(await screen.findByText('Not asked yet')).toBeTruthy();
  });
});

describe('general', () => {
  it('turns launch at login on and off', async () => {
    const world = install();
    const user = userEvent.setup();
    renderSettings();
    const toggle = await screen.findByRole('switch', { name: 'Launch at login' });
    expect(toggle.getAttribute('aria-checked')).toBe('false');

    await user.click(toggle);

    expect(invokeMock).toHaveBeenCalledWith('update_settings', {
      settings: { ...DEFAULT_SETTINGS, launchAtLogin: true },
    });
    await waitFor(() => {
      expect(
        screen.getByRole('switch', { name: 'Launch at login' }).getAttribute('aria-checked'),
      ).toBe('true');
    });
    expect(world.settings.launchAtLogin).toBe(true);
  });

  it('updates automatically by default and can be turned off', async () => {
    const world = install();
    const user = userEvent.setup();
    renderSettings();
    const toggle = await screen.findByRole('switch', { name: 'Update automatically' });
    expect(toggle.getAttribute('aria-checked')).toBe('true');

    await user.click(toggle);

    expect(invokeMock).toHaveBeenCalledWith('update_settings', {
      settings: { ...DEFAULT_SETTINGS, autoUpdate: false },
    });
    await waitFor(() => {
      expect(world.settings.autoUpdate).toBe(false);
    });
  });

  it('shows why launch at login could not be changed', async () => {
    install(
      {},
      {
        update_settings: () => {
          throw new CommandFailure('platform', 'The login item could not be written');
        },
      },
    );
    const user = userEvent.setup();
    renderSettings();

    await user.click(await screen.findByRole('switch', { name: 'Launch at login' }));

    expect((await screen.findByRole('alert')).textContent).toBe(
      "Launch at login couldn't be changed. macOS didn't allow that.",
    );
    expect(
      screen.getByRole('switch', { name: 'Launch at login' }).getAttribute('aria-checked'),
    ).toBe('false');
  });
});

describe('history', () => {
  function retention(): HTMLSelectElement {
    const select = screen.getByRole('combobox', { name: 'Keep history for' });
    if (!(select instanceof HTMLSelectElement)) throw new Error('Not a select');
    return select;
  }

  it('offers 30, 90, 180 and 365 days in a pop-up menu with the current one selected', async () => {
    install();
    renderSettings();

    await screen.findByRole('combobox', { name: 'Keep history for' });

    expect(
      within(retention())
        .getAllByRole('option')
        .map((option) => option.textContent),
    ).toEqual(['30 days', '90 days', '180 days', '1 year']);
    expect(retention().value).toBe('90');
  });

  it('says older history is removed automatically, not on a schedule it cannot promise', async () => {
    install();
    renderSettings();

    expect(await screen.findByText('Older history is removed automatically.')).toBeTruthy();
  });

  it('keeps an unusual current value visible', async () => {
    install({ settings: { ...DEFAULT_SETTINGS, retentionDays: 60 } });
    renderSettings();

    await screen.findByRole('combobox', { name: 'Keep history for' });

    expect(retention().value).toBe('60');
    expect(screen.getByRole('option', { name: '60 days' })).toBeTruthy();
  });

  it('saves a longer period at once, since nothing is deleted', async () => {
    install();
    const user = userEvent.setup();
    renderSettings();
    await screen.findByRole('combobox', { name: 'Keep history for' });

    await user.selectOptions(retention(), '180');

    expect(invokeMock).toHaveBeenCalledWith('update_settings', {
      settings: { ...DEFAULT_SETTINGS, retentionDays: 180 },
    });
    await waitFor(() => {
      expect(retention().value).toBe('180');
    });
    expect(screen.queryByRole('dialog')).toBeNull();
  });

  it('asks before shortening the period and saves nothing until confirmed', async () => {
    const world = install();
    const user = userEvent.setup();
    renderSettings();
    await screen.findByRole('combobox', { name: 'Keep history for' });

    await user.selectOptions(retention(), '30');

    const dialog = await screen.findByRole('dialog', { name: 'Keep a shorter history?' });
    expect(
      within(dialog).getByText("History older than 30 days is deleted now. This can't be undone."),
    ).toBeTruthy();
    expect(invokeMock).not.toHaveBeenCalledWith('update_settings', expect.anything());
    expect(world.settings.retentionDays).toBe(90);

    await user.click(within(dialog).getByRole('button', { name: 'Delete older history' }));

    expect(invokeMock).toHaveBeenCalledWith('update_settings', {
      settings: { ...DEFAULT_SETTINGS, retentionDays: 30 },
    });
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
    expect(retention().value).toBe('30');
  });

  it('names a year as a year when shortening to it', async () => {
    install({ settings: { ...DEFAULT_SETTINGS, retentionDays: 365 } });
    const user = userEvent.setup();
    renderSettings();
    await screen.findByRole('combobox', { name: 'Keep history for' });

    await user.selectOptions(retention(), '180');

    expect(
      await screen.findByText("History older than 180 days is deleted now. This can't be undone."),
    ).toBeTruthy();
  });

  it('goes back to the saved period when the shortening is cancelled', async () => {
    install();
    const user = userEvent.setup();
    renderSettings();
    await screen.findByRole('combobox', { name: 'Keep history for' });
    await user.selectOptions(retention(), '30');
    const dialog = await screen.findByRole('dialog');

    await user.click(within(dialog).getByRole('button', { name: 'Cancel' }));

    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
    expect(retention().value).toBe('90');
    expect(invokeMock).not.toHaveBeenCalledWith('update_settings', expect.anything());
  });

  it('holds the sheet open and explains when shortening fails', async () => {
    install(
      {},
      {
        update_settings: () => {
          throw new CommandFailure('storage', 'The database is locked');
        },
      },
    );
    const user = userEvent.setup();
    renderSettings();
    await screen.findByRole('combobox', { name: 'Keep history for' });
    await user.selectOptions(retention(), '30');
    const dialog = await screen.findByRole('dialog');

    await user.click(within(dialog).getByRole('button', { name: 'Delete older history' }));

    expect((await within(dialog).findByRole('alert')).textContent).toBe(
      "The history length wasn't changed. Grumpo couldn't read or save its data on this Mac.",
    );
    expect(screen.getByRole('dialog')).toBeTruthy();
  });

  it('cannot be dismissed while the shorter period is being saved', async () => {
    let finish: () => void = () => undefined;
    install(
      {},
      {
        update_settings: ({ settings }) =>
          new Promise<Settings>((resolve) => {
            finish = () => {
              resolve(settings);
            };
          }),
      },
    );
    const user = userEvent.setup();
    renderSettings();
    await screen.findByRole('combobox', { name: 'Keep history for' });
    await user.selectOptions(retention(), '30');
    const dialog = await screen.findByRole('dialog');
    await user.click(within(dialog).getByRole('button', { name: 'Delete older history' }));

    fireEvent.keyDown(dialog, { key: 'Escape' });

    expect(screen.getByRole('dialog')).toBeTruthy();
    finish();
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
  });

  it('shows why a longer period could not be saved and returns to the saved one', async () => {
    install(
      {},
      {
        update_settings: () => {
          throw new CommandFailure('storage', 'The database is locked');
        },
      },
    );
    const user = userEvent.setup();
    renderSettings();
    await screen.findByRole('combobox', { name: 'Keep history for' });

    await user.selectOptions(retention(), '365');

    expect((await screen.findByRole('alert')).textContent).toContain(
      "Grumpo couldn't read or save its data on this Mac.",
    );
    expect(retention().value).toBe('90');
  });

  it('exports and tells you where the file went', async () => {
    install();
    const user = userEvent.setup();
    renderSettings();

    await user.click(await screen.findByRole('button', { name: 'Export data' }));

    expect(invokeMock).toHaveBeenCalledWith('export_data');
    const toast = await screen.findByText(
      'Exported to Downloads as Focus export 2026-10-12 1530.json',
    );
    expect(toast.closest('[aria-live="polite"]')).not.toBeNull();
  });

  it('dismisses the toast', async () => {
    install();
    const user = userEvent.setup();
    renderSettings();
    await user.click(await screen.findByRole('button', { name: 'Export data' }));
    await screen.findByText(/Exported to Downloads/);

    await user.click(screen.getByRole('button', { name: 'Dismiss' }));

    await waitFor(() => {
      expect(screen.queryByText(/Exported to Downloads/)).toBeNull();
    });
  });

  it('shows why an export failed', async () => {
    install(
      {},
      {
        export_data: () => {
          throw new CommandFailure('storage', 'The Downloads folder is not writable');
        },
      },
    );
    const user = userEvent.setup();
    renderSettings();

    await user.click(await screen.findByRole('button', { name: 'Export data' }));

    expect((await screen.findByRole('alert')).textContent).toContain(
      "The export didn't finish. Grumpo couldn't read or save its data on this Mac.",
    );
    expect(screen.queryByText(/Exported to Downloads/)).toBeNull();
  });
});

describe('delete history', () => {
  async function openSheet() {
    const user = userEvent.setup();
    renderSettings();
    await user.click(await screen.findByRole('button', { name: 'Delete history' }));
    const dialog = await screen.findByRole('dialog', { name: 'Delete history?' });
    return { user, dialog };
  }

  it('says exactly what goes and what stays, and nothing is deleted yet', async () => {
    install();
    const { dialog } = await openSheet();

    expect(
      within(dialog).getByText(
        /every recorded session, which is all the time tracked per app and site/,
      ),
    ).toBeTruthy();
    expect(
      within(dialog).getByText(/Your rules, schedule, daily limits and settings stay/),
    ).toBeTruthy();
    expect(
      within(dialog).getByText(
        /Today's history is kept until midnight, because your daily limits count it/,
      ),
    ).toBeTruthy();
    expect(within(dialog).getByText("This can't be undone.")).toBeTruthy();
    expect(invokeMock).not.toHaveBeenCalledWith('delete_all_data');
  });

  it('says today is deleted too when no limit needs it', async () => {
    install({ status: connectedStatus({ rules: EMPTY_RULES }) });
    const { dialog } = await openSheet();

    expect(await within(dialog).findByText(/Today's history is deleted too/)).toBeTruthy();
  });

  it('keeps today and says why when the service cannot say whether limits exist', async () => {
    install({ status: NOT_INSTALLED });
    const { dialog } = await openSheet();

    expect(
      within(dialog).getByText(
        "Today's history is kept until midnight because Grumpo can't check your daily limits right now.",
      ),
    ).toBeTruthy();
    expect(within(dialog).queryByText(/because your daily limits count it/)).toBeNull();
  });

  it('cancels without deleting', async () => {
    install();
    const { user, dialog } = await openSheet();

    await user.click(within(dialog).getByRole('button', { name: 'Cancel' }));

    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
    expect(invokeMock).not.toHaveBeenCalledWith('delete_all_data');
  });

  it('deletes, closes and confirms with a toast', async () => {
    install();
    const { user, dialog } = await openSheet();

    await user.click(within(dialog).getByRole('button', { name: 'Delete history' }));

    expect(invokeMock).toHaveBeenCalledWith('delete_all_data');
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
    expect(await screen.findByText('History deleted')).toBeTruthy();
  });

  it('stays open and explains when deleting fails', async () => {
    install(
      {},
      {
        delete_all_data: () => {
          throw new CommandFailure('storage', 'The database is locked');
        },
      },
    );
    const { user, dialog } = await openSheet();

    await user.click(within(dialog).getByRole('button', { name: 'Delete history' }));

    const alert = await within(dialog).findByRole('alert');
    expect(alert.textContent).toBe(
      "Nothing was deleted. Grumpo couldn't read or save its data on this Mac.",
    );
    expect(screen.getByRole('dialog')).toBeTruthy();
  });

  it('cannot be dismissed while it deletes', async () => {
    let finish: () => void = () => undefined;
    install(
      {},
      {
        delete_all_data: () =>
          new Promise<null>((resolve) => {
            finish = () => {
              resolve(null);
            };
          }),
      },
    );
    const { user, dialog } = await openSheet();
    await user.click(within(dialog).getByRole('button', { name: 'Delete history' }));

    fireEvent.keyDown(dialog, { key: 'Escape' });
    expect(screen.getByRole('dialog')).toBeTruthy();
    expect(within(dialog).getByRole('button', { name: 'Cancel' }).hasAttribute('disabled')).toBe(
      true,
    );

    finish();
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
  });

  it('has no accessibility violations', async () => {
    install();
    await openSheet();

    await expectAccessible(document.body);
  });
});

describe('loading and failing', () => {
  it('shows the settings failure with a retry', async () => {
    let healthy = false;
    install(
      {},
      {
        get_settings: () => {
          if (!healthy) throw new CommandFailure('storage', 'Settings file is unreadable');
          return DEFAULT_SETTINGS;
        },
      },
    );
    const user = userEvent.setup();
    renderSettings();

    expect(
      await screen.findByText("Grumpo couldn't read or save its data on this Mac."),
    ).toBeTruthy();
    expect(screen.queryByText('Settings file is unreadable')).toBeNull();
    healthy = true;
    await user.click(within(section('General')).getByRole('button', { name: 'Try again' }));

    expect(await screen.findByRole('switch', { name: 'Launch at login' })).toBeTruthy();
  });
});

describe('accessibility', () => {
  it('has no violations with everything loaded', async () => {
    install();
    const { container } = renderSettings();
    await screen.findByRole('switch', { name: 'Launch at login' });

    await expectAccessible(container, true);
  });

  it('has no violations with the service missing', async () => {
    install({ status: NOT_INSTALLED });
    const { container } = renderSettings();
    await screen.findByRole('button', { name: 'Install service' });

    await expectAccessible(container, true);
  });

  it('has no violations with a failed install and the Terminal command open', async () => {
    install(
      { status: NOT_INSTALLED },
      {
        install_service: () => {
          throw new CommandFailure('serviceInstallFailed', 'exit 1');
        },
      },
    );
    const user = userEvent.setup();
    const { container } = renderSettings();
    await user.click(await screen.findByRole('button', { name: 'Install service' }));
    await user.click(await screen.findByText('Install from Terminal instead'));

    await expectAccessible(container, true);
  });
});

describe('explanations', () => {
  it('says what each permission is for on the row and keeps the reasons behind a disclosure', async () => {
    install();
    const user = userEvent.setup();
    renderSettings();

    const chrome = within(
      await screen.findByText('Google Chrome').then(() => rowOf('Google Chrome')),
    );
    expect(chrome.getByText(/close a tab on a blocked site, in the frontmost window/)).toBeTruthy();
    expect(chrome.queryByText(/Blocked tabs stay open/)).toBeNull();

    await user.click(chrome.getByRole('button', { name: 'Why Grumpo needs this' }));

    expect(await chrome.findByText(/Blocked tabs stay open in Google Chrome/)).toBeTruthy();
    expect(chrome.getByText('How to allow it')).toBeTruthy();
    expect(chrome.getByText(/it never reads page content/i)).toBeTruthy();
  });

  it('explains the service the same way, whether or not it is running', async () => {
    install();
    const user = userEvent.setup();
    renderSettings();
    await screen.findByText('Running');

    await user.click(firstOf(screen.getAllByRole('button', { name: 'Why Grumpo needs this' })));

    expect(await screen.findByText(/never sees your password/)).toBeTruthy();
    expect(screen.getByText(/Nothing gets blocked/)).toBeTruthy();
  });

  it('reopens the welcome from General', async () => {
    install({}, { reset_onboarding: () => null });
    const user = userEvent.setup();
    renderSettings();

    await user.click(await screen.findByRole('button', { name: 'Show welcome again' }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('reset_onboarding');
    });
  });

  it('says so when the welcome cannot be reopened', async () => {
    install(
      {},
      {
        reset_onboarding: () => {
          throw new CommandFailure('platform', 'nope');
        },
      },
    );
    const user = userEvent.setup();
    renderSettings();

    await user.click(await screen.findByRole('button', { name: 'Show welcome again' }));

    expect((await screen.findByRole('alert')).textContent).toContain(
      "The welcome couldn't be reopened.",
    );
  });
});

function rowOf(name: string): HTMLElement {
  const row = screen.getAllByRole('listitem').find((item) => item.textContent.startsWith(name));
  if (row === undefined) throw new Error(`No row for ${name}`);
  return row;
}

function firstOf(items: HTMLElement[]): HTMLElement {
  const [first] = items;
  if (first === undefined) throw new Error('Expected at least one element');
  return first;
}
