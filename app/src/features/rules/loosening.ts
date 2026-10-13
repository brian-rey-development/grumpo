import { rulesCopy } from '@/features/rules/copy';
import { minutesLost } from '@/features/rules/coverage';
import type { RulesLocks } from '@/features/status/useRulesLocks';
import { HOST_ALIAS_PREFIXES } from '@/ipc/generated/constants';
import type { LimitTarget } from '@/ipc/generated/LimitTarget';
import type { Rules } from '@/ipc/generated/Rules';
import { formatDurationLong } from '@/lib/format';
import { targetKey, targetName } from '@/lib/limitTarget';
import { SECONDS_PER_MINUTE } from '@/lib/time';

const MAX_NAMED = 2;
const copy = rulesCopy.loosening;

type Loosening =
  | { kind: 'siteRemoved'; domain: string }
  | { kind: 'appRemoved'; name: string }
  | { kind: 'limitRemoved'; target: LimitTarget }
  | { kind: 'limitRaised'; target: LimitTarget; from: number; to: number }
  | { kind: 'scheduleShortened'; minutesPerWeek: number };

/** True when blocking `kept` still blocks `domain`, which is its own host and the aliases the service adds. */
function stillBlockedBy(kept: string, domain: string): boolean {
  return HOST_ALIAS_PREFIXES.some((prefix) => domain === `${prefix}${kept}`);
}

/** The ways `draft` blocks less than `saved`, which the service refuses while focus is on. */
function findLoosenings(saved: Rules, draft: Rules): Loosening[] {
  const found: Loosening[] = [];
  for (const domain of saved.blockedSites) {
    if (!draft.blockedSites.some((kept) => stillBlockedBy(kept, domain))) {
      found.push({ kind: 'siteRemoved', domain });
    }
  }
  for (const app of saved.blockedApps) {
    if (!draft.blockedApps.some((kept) => kept.bundleId === app.bundleId)) {
      found.push({ kind: 'appRemoved', name: app.name });
    }
  }
  for (const limit of saved.limits) {
    const now = draft.limits.find((entry) => targetKey(entry.target) === targetKey(limit.target));
    if (now === undefined) found.push({ kind: 'limitRemoved', target: limit.target });
    else if (now.dailyMinutes > limit.dailyMinutes) {
      found.push({
        kind: 'limitRaised',
        target: limit.target,
        from: limit.dailyMinutes,
        to: now.dailyMinutes,
      });
    }
  }
  const lost = minutesLost(saved.schedule, draft.schedule);
  if (lost > 0) found.push({ kind: 'scheduleShortened', minutesPerWeek: lost });
  return found;
}

function nameList(names: readonly string[], noun: string): string {
  const [first = '', second = ''] = names;
  if (names.length <= MAX_NAMED) return names.length === 2 ? copy.pair(first, second) : first;
  return copy.others(first, names.length - 1, noun);
}

function minutesText(minutes: number): string {
  return formatDurationLong(minutes * SECONDS_PER_MINUTE);
}

function limitPhrases(found: readonly Loosening[]): string[] {
  const removed = found.flatMap((item) => (item.kind === 'limitRemoved' ? [item.target] : []));
  const raised = found.flatMap((item) => (item.kind === 'limitRaised' ? [item] : []));
  const phrases: string[] = [];
  const [firstRemoved, secondRemoved] = removed;
  if (removed.length === 1 && firstRemoved)
    phrases.push(copy.removeOneLimit(targetName(firstRemoved)));
  else if (removed.length === 2 && firstRemoved && secondRemoved) {
    phrases.push(copy.removeTwoLimits(targetName(firstRemoved), targetName(secondRemoved)));
  } else if (removed.length > 2) phrases.push(copy.removeManyLimits(removed.length));
  const [firstRaised, secondRaised] = raised;
  if (raised.length === 1 && firstRaised) {
    const { target, from, to } = firstRaised;
    phrases.push(copy.raiseOneLimit(targetName(target), minutesText(from), minutesText(to)));
  } else if (raised.length === 2 && firstRaised && secondRaised) {
    phrases.push(
      copy.raiseTwoLimits(targetName(firstRaised.target), targetName(secondRaised.target)),
    );
  } else if (raised.length > 2) phrases.push(copy.raiseManyLimits(raised.length));
  return phrases;
}

function phrasesFor(found: readonly Loosening[]): string[] {
  const sites = found.flatMap((item) => (item.kind === 'siteRemoved' ? [item.domain] : []));
  const apps = found.flatMap((item) => (item.kind === 'appRemoved' ? [item.name] : []));
  const schedule = found.find((item) => item.kind === 'scheduleShortened');
  return [
    ...(sites.length > 0 ? [copy.removeSites(nameList(sites, 'site'))] : []),
    ...(apps.length > 0 ? [copy.stopApps(nameList(apps, 'app'))] : []),
    ...limitPhrases(found),
    ...(schedule ? [copy.shortenSchedule(minutesText(schedule.minutesPerWeek))] : []),
  ];
}

function lowerFirst(text: string): string {
  return text.charAt(0).toLowerCase() + text.slice(1);
}

function joinPhrases(phrases: readonly string[]): string {
  const [first = '', ...rest] = phrases;
  const tail = rest.map(lowerFirst);
  const last = tail.pop();
  if (last === undefined) return `${first}.`;
  return `${[first, ...tail].join(', ')} and ${last}.`;
}

function isLocked(item: Loosening, locks: RulesLocks): boolean {
  return item.kind === 'limitRemoved' || item.kind === 'limitRaised'
    ? locks.limits
    : locks.blocking;
}

function lockedLoosenings(saved: Rules, draft: Rules, locks: RulesLocks): Loosening[] {
  return findLoosenings(saved, draft).filter((item) => isLocked(item, locks));
}

/** True when a locked loosening raises or removes a daily limit, which needs an unlock whatever the schedule. */
export function includesLockedLimitChange(saved: Rules, draft: Rules, locks: RulesLocks): boolean {
  return lockedLoosenings(saved, draft, locks).some(
    (item) => item.kind === 'limitRemoved' || item.kind === 'limitRaised',
  );
}

/**
 * One plain sentence naming what the draft loosens that is locked right now, or null when nothing
 * is. Limits are locked whatever the schedule, and the rest only while blocking is in force.
 */
export function describeLoosening(saved: Rules, draft: Rules, locks: RulesLocks): string | null {
  const locked = lockedLoosenings(saved, draft, locks);
  const phrases = phrasesFor(locked);
  return phrases.length === 0 ? null : joinPhrases(phrases);
}
