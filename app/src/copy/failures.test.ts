import { describe, expect, it } from 'vitest';

import { describeFailure } from '@/copy/failures';
import { CommandFailure, UNKNOWN_ERROR_CODE } from '@/ipc/errors';
import type { CommandErrorCode } from '@/ipc/generated/CommandErrorCode';
import type { RulesFault } from '@/ipc/generated/RulesFault';

const ALL_CODES = {
  daemonUnavailable: "The Grumpo service isn't running",
  versionMismatch: 'Grumpo and its service are on different versions',
  rulesInvalid: "Grumpo can't use these rules",
  looseningLocked: 'This change needs an unlock first',
  unlockNotReady: "The wait isn't over yet",
  unlockNotRequested: 'No unlock is waiting',
  unlockAlreadyPending: 'An unlock is already waiting',
  unlockAlreadyActive: 'Focus is already unlocked',
  phraseMismatch: "That doesn't match the sentence",
  durationInvalid: "That length isn't available",
  invalidInput: "Grumpo didn't accept that",
  storage: "Grumpo couldn't read or save its data",
  exportDenied: "macOS didn't let Grumpo save the export",
  serviceInstallUnavailable: "This copy of Grumpo can't install the service by itself",
  serviceInstallFailed: "The Grumpo service couldn't be installed",
  serviceInstallBusy: 'An install is already running',
  platform: "macOS didn't allow that",
  internal: 'Something went wrong inside Grumpo',
} as const satisfies Record<CommandErrorCode, string>;

describe('describeFailure', () => {
  it.each(Object.entries(ALL_CODES))('explains %s in plain words', (code, start) => {
    const failure = new CommandFailure(code as CommandErrorCode, 'raw backend text');

    expect(describeFailure(failure, 'fallback').startsWith(start)).toBe(true);
  });

  it('never shows the message the backend sent', () => {
    const failure = new CommandFailure('storage', 'Disk is full at /Users/someone/focus.db');

    expect(describeFailure(failure, 'fallback')).not.toContain('Disk is full');
  });

  it('uses the fallback for a rejection without a known code', () => {
    const failure = new CommandFailure(UNKNOWN_ERROR_CODE, 'socket closed');

    expect(describeFailure(failure, 'Try again in a moment.')).toBe('Try again in a moment.');
  });

  it('uses no colons and no contractions that spell the words out', () => {
    for (const code of Object.keys(ALL_CODES)) {
      const text = describeFailure(new CommandFailure(code as CommandErrorCode, ''), '');
      expect(text).not.toContain(':');
      expect(text).not.toMatch(/\b(cannot|could not|does not|is not)\b/);
    }
  });
});

const SITE = { kind: 'site', domain: 'youtube.com' } as const;

describe('describeFailure for refused rules', () => {
  function refused(rulesFault: RulesFault | null): string {
    const failure = new CommandFailure('rulesInvalid', 'service text', { rulesFault });
    return describeFailure(failure, 'fallback');
  }

  it.each<[RulesFault, string]>([
    [{ kind: 'unreadable' }, "Grumpo couldn't read these rules. Try saving them again."],
    [
      { kind: 'invalidSite', site: 'co.uk', problem: 'publicSuffix' },
      "co.uk can't be blocked. That ending is shared by many sites, like co.uk. Enter a specific site under it.",
    ],
    [
      { kind: 'invalidSite', site: 'localhost', problem: 'tooFewLabels' },
      "localhost can't be blocked. Enter a full address with a dot, like youtube.com.",
    ],
    [
      { kind: 'invalidApp' },
      "One of the apps on the list isn't valid. Remove it and add it again.",
    ],
    [{ kind: 'tooMany', section: 'sites' }, 'There are too many sites. Grumpo keeps at most 500.'],
    [
      { kind: 'tooMany', section: 'windows' },
      'There are too many focus times. Grumpo keeps at most 50.',
    ],
    [{ kind: 'windowWithoutDays', number: 3 }, 'Focus time 3 needs at least one day.'],
    [{ kind: 'windowNotForward', number: 1 }, 'Focus time 1 has to end after it starts.'],
    [
      { kind: 'limitOutOfRange', target: SITE, minutes: 2000 },
      "The limit for youtube.com has to be between 1 minute and 24 hours, so 33 hours 20 minutes isn't allowed.",
    ],
    [{ kind: 'duplicateLimit', target: SITE }, "There's already a limit for youtube.com."],
    [
      { kind: 'protectedApp', bundleId: 'com.apple.finder' },
      "Grumpo can't block com.apple.finder, because the Mac needs it to work.",
    ],
  ])('words the fault %j specifically', (fault, expected) => {
    expect(refused(fault)).toBe(expected);
  });

  it('falls back to the general wording without a fault, and never shows the service text', () => {
    expect(refused(null)).toBe(
      "Grumpo can't use these rules. Check the sites, apps and focus times, then try again.",
    );
    expect(refused({ kind: 'invalidApp' })).not.toContain('service text');
  });

  it('only reads the fault of refused rules', () => {
    const failure = new CommandFailure('storage', 'x', { rulesFault: { kind: 'unreadable' } });

    expect(describeFailure(failure, 'fallback')).toContain("couldn't read or save its data");
  });
});
