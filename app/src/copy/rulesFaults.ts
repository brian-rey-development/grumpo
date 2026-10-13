import {
  MAX_APPS,
  MAX_LABEL_LEN,
  MAX_LIMIT_MINUTES,
  MAX_LIMITS,
  MAX_SITES,
  MAX_WINDOWS,
  MIN_LIMIT_MINUTES,
} from '@/ipc/generated/constants';
import type { LimitTarget } from '@/ipc/generated/LimitTarget';
import type { RuleSection } from '@/ipc/generated/RuleSection';
import type { RulesFault } from '@/ipc/generated/RulesFault';
import type { SiteProblem } from '@/ipc/generated/SiteProblem';
import { formatDurationLong } from '@/lib/format';
import { targetName } from '@/lib/limitTarget';
import { SECONDS_PER_MINUTE } from '@/lib/time';

export const siteProblemCopy: Record<SiteProblem, string> = {
  empty: 'Enter a site like youtube.com.',
  nonAscii: 'Use only plain letters and numbers in the site name.',
  tooLong: 'That address is too long.',
  tooFewLabels: 'Enter a full address with a dot, like youtube.com.',
  emptyLabel: 'Remove the doubled dots from the address.',
  labelTooLong: `Each part of the address can have at most ${MAX_LABEL_LEN.toString()} characters.`,
  invalidCharacter: 'Use only letters, numbers, dots and hyphens.',
  hyphenAtEdge: "A part of the address can't start or end with a hyphen.",
  ipAddress: "IP addresses can't be blocked here. Enter a site name.",
  publicSuffix: 'That ending is shared by many sites, like co.uk. Enter a specific site under it.',
};

export function duplicateLimitCopy(name: string): string {
  return `There's already a limit for ${name}.`;
}

const SECTION_LIMITS: Record<RuleSection, { noun: string; max: number }> = {
  sites: { noun: 'sites', max: MAX_SITES },
  apps: { noun: 'apps', max: MAX_APPS },
  windows: { noun: 'focus times', max: MAX_WINDOWS },
  limits: { noun: 'daily limits', max: MAX_LIMITS },
};

function minutesText(minutes: number): string {
  return formatDurationLong(minutes * SECONDS_PER_MINUTE);
}

function limitRangeCopy(target: LimitTarget, minutes: number): string {
  const range = `${minutesText(MIN_LIMIT_MINUTES)} and ${minutesText(MAX_LIMIT_MINUTES)}`;
  return `The limit for ${targetName(target)} has to be between ${range}, so ${minutesText(minutes)} isn't allowed.`;
}

/** What was wrong with the rules, in a sentence that says what to change. */
export function describeRulesFault(fault: RulesFault): string {
  switch (fault.kind) {
    case 'unreadable':
      return "Grumpo couldn't read these rules. Try saving them again.";
    case 'invalidSite':
      return `${fault.site} can't be blocked. ${siteProblemCopy[fault.problem]}`;
    case 'invalidApp':
      return "One of the apps on the list isn't valid. Remove it and add it again.";
    case 'tooMany': {
      const { noun, max } = SECTION_LIMITS[fault.section];
      return `There are too many ${noun}. Grumpo keeps at most ${max.toString()}.`;
    }
    case 'windowWithoutDays':
      return `Focus time ${fault.number.toString()} needs at least one day.`;
    case 'windowNotForward':
      return `Focus time ${fault.number.toString()} has to end after it starts.`;
    case 'limitOutOfRange':
      return limitRangeCopy(fault.target, fault.minutes);
    case 'duplicateLimit':
      return duplicateLimitCopy(targetName(fault.target));
    case 'protectedApp':
      return `Grumpo can't block ${fault.bundleId}, because the Mac needs it to work.`;
  }
}
