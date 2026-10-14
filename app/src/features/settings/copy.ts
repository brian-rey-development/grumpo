import type { BrowserId } from '@/ipc/generated/BrowserId';
import type { PermissionStatus } from '@/ipc/generated/PermissionStatus';

export const settingsCopy = {
  loadFailed: "Settings couldn't be loaded.",
  service: {
    title: 'Grumpo service',
    description:
      'A small background service does the blocking, because only it is allowed to change the list of blocked sites.',
    running: 'Running',
    runningDetail: 'The Grumpo service and this app are on the same version.',
    install: {
      notInstalled: 'Install service',
      serviceOutdated: 'Update service',
    },
    passwordNote: {
      notInstalled: 'macOS will ask for your Mac password once, to install the service.',
      serviceOutdated: 'macOS will ask for your Mac password once, to update the service.',
    },
    progress: {
      notInstalled: 'Installing the Grumpo service. Enter your Mac password if macOS asks.',
      serviceOutdated: 'Updating the Grumpo service. Enter your Mac password if macOS asks.',
    },
    terminalInstead: 'Install from Terminal instead',
    commandLabel: 'Install command',
    checkAgain: 'Check again',
    stillDown: "The Grumpo service still isn't running.",
    copy: 'Copy',
    copied: 'Copied',
    copyFailed: "The command couldn't be copied. Select it and copy it by hand.",
  },
  permissions: {
    title: 'Permissions',
    description:
      "Grumpo asks for these so it can see which app and site you're in. Nothing leaves your Mac.",
    accessibility: 'Accessibility',
    allow: 'Allow',
    openSettings: 'Open System Settings',
    states: {
      granted: 'Allowed',
      denied: 'Not allowed',
      notDetermined: 'Not asked yet',
      unsupported: 'Not available on this Mac',
    } satisfies Record<Exclude<PermissionStatus, 'notRunning'>, string>,
    notRunning: (browser: string) => `Open ${browser} to check access.`,
    deniedHint: {
      browser: 'Turn Grumpo on under Privacy and Security, then Automation.',
      accessibility:
        'Already on in System Settings? Choose Allow, and Grumpo replaces the entry an older version left behind.',
    },
    firefox: {
      covered: 'Covered by Accessibility',
      needsAccessibility: 'Needs Accessibility',
      detail:
        "Grumpo reads the site of the tab you're on in Firefox through Accessibility, so there's nothing else to allow. It can't close Firefox tabs, but blocked sites still won't load.",
    },
    loadFailed: "Grumpo couldn't read its permissions.",
    actionFailed: (name: string) => `${name} couldn't be changed.`,
  },
  browsers: {
    chrome: 'Google Chrome',
    brave: 'Brave',
    edge: 'Microsoft Edge',
    safari: 'Safari',
    firefox: 'Firefox',
  } satisfies Record<BrowserId, string>,
  general: {
    title: 'General',
    launchAtLogin: 'Launch at login',
    launchAtLoginDetail: 'Open Grumpo when you log in, so blocking and tracking start right away.',
    launchAtLoginFailed: "Launch at login couldn't be changed.",
    autoUpdate: 'Update automatically',
    autoUpdateDetail:
      'Once a day, Grumpo asks GitHub for a newer version and installs it. Nothing about you is sent.',
    autoUpdateFailed: "Automatic updates couldn't be changed.",
    welcome: 'Welcome',
    welcomeDetail: 'Walk through what Grumpo does and why he needs access, one more time.',
    showWelcome: 'Show welcome again',
    welcomeFailed: "The welcome couldn't be reopened.",
  },
  history: {
    title: 'History',
    description: 'Older history is removed automatically.',
    keepFor: 'Keep history for',
    keepForDetail: 'Reports reach back this far.',
    retentionFailed: "The history length couldn't be changed.",
    export: 'Export data',
    exportDetail: 'Saves every tracked session and your settings as a JSON file in Downloads.',
    exporting: 'Exporting',
    exported: (fileName: string) => `Exported to Downloads as ${fileName}`,
    exportFailed: "The export didn't finish.",
    delete: 'Delete history',
    deleteDetail: 'Removes your tracked history. Your rules and settings stay.',
    deleted: 'History deleted',
  },
  retentionSheet: {
    title: 'Keep a shorter history?',
    older: (period: string) => `History older than ${period} is deleted now. This can't be undone.`,
    confirm: 'Delete older history',
    failed: "The history length wasn't changed.",
  },
  deleteSheet: {
    title: 'Delete history?',
    what: 'This permanently deletes every recorded session, which is all the time tracked per app and site, and the reports built from it.',
    kept: 'Your rules, schedule, daily limits and settings stay as they are.',
    todayKept:
      "Today's history is kept until midnight, because your daily limits count it. Deleting it would hand back the time you already spent.",
    todayDeleted: "Today's history is deleted too, since no daily limit needs it.",
    todayKeptUnknown:
      "Today's history is kept until midnight because Grumpo can't check your daily limits right now.",
    final: "This can't be undone.",
    confirm: 'Delete history',
    failed: 'Nothing was deleted.',
  },
} as const;
