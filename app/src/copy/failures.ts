import { describeRulesFault } from '@/copy/rulesFaults';
import { type CommandFailure, UNKNOWN_ERROR_CODE } from '@/ipc/errors';
import type { CommandErrorCode } from '@/ipc/generated/CommandErrorCode';

const FAILURE_COPY: Record<CommandErrorCode, string> = {
  daemonUnavailable: "The Grumpo service isn't running. Open Settings to fix it.",
  versionMismatch:
    'Grumpo and its service are on different versions, so one needs updating. Open Settings to see how.',
  rulesInvalid:
    "Grumpo can't use these rules. Check the sites, apps and focus times, then try again.",
  looseningLocked: 'This change needs an unlock first.',
  unlockNotReady: "The wait isn't over yet. Grumpo will tell you when it's ready.",
  unlockNotRequested: 'No unlock is waiting. Start the wait again.',
  unlockAlreadyPending: 'An unlock is already waiting.',
  unlockAlreadyActive: 'Focus is already unlocked.',
  phraseMismatch:
    "That doesn't match the sentence. Check the words, capital letters and punctuation.",
  durationInvalid: "That length isn't available. Choose one of the options.",
  invalidInput: "Grumpo didn't accept that. Check it and try again.",
  storage: "Grumpo couldn't read or save its data on this Mac.",
  exportDenied:
    "macOS didn't let Grumpo save the export. Allow Grumpo to use your Downloads folder in System Settings, then try again.",
  serviceInstallUnavailable: "This copy of Grumpo can't install the service by itself.",
  serviceInstallFailed:
    "The Grumpo service couldn't be installed. Try again, or install it from Terminal instead.",
  serviceInstallBusy: 'An install is already running. Wait for it to finish, then check Settings.',
  platform: "macOS didn't allow that.",
  internal: 'Something went wrong inside Grumpo.',
};

/**
 * The wording for a failed command, chosen from its code, or from the specific fault when rules
 * were refused. The service's own message is for logs, so it is never shown, and `fallback`
 * covers a rejection that carries no known code.
 */
export function describeFailure(failure: CommandFailure, fallback: string): string {
  if (failure.code === UNKNOWN_ERROR_CODE) return fallback;
  if (failure.code === 'rulesInvalid' && failure.rulesFault !== null) {
    return describeRulesFault(failure.rulesFault);
  }
  return FAILURE_COPY[failure.code];
}
