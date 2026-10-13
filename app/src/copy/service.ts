import type { DaemonUnavailableReason } from '@/ipc/generated/DaemonUnavailableReason';

export const INSTALL_COMMAND =
  'scripts/build-release.sh && sudo scripts/install.sh --binary dist/release/focusd';

export const serviceCopy = {
  reasons: {
    notInstalled: "The Grumpo service isn't installed, so nothing can be blocked yet.",
    notResponding:
      "The Grumpo service is installed but isn't answering. macOS restarts it on its own, so check again in a moment.",
    serviceOutdated: 'The Grumpo service is older than this app and needs updating.',
    appOutdated: 'Grumpo needs updating. The Grumpo service on this Mac is newer than this app.',
    otherUser: 'The Grumpo service on this Mac belongs to another user.',
  } satisfies Record<DaemonUnavailableReason, string>,
  runCommand:
    'Run this command in Terminal from the Grumpo project folder, then choose Check again.',
};
