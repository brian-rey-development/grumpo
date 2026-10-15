import { describe, expect, it } from 'vitest';

import {
  describeLoosening as describeLocked,
  includesLockedLimitChange,
} from '@/features/rules/loosening';
import type { RulesLocks } from '@/features/status/useRulesLocks';
import {
  addApp,
  addLimit,
  addSite,
  addWindow,
  removeApp,
  removeLimit,
  removeSite,
  setLimitMinutes,
  updateWindow,
} from '@/features/rules/rulesModel';
import type { Rules } from '@/ipc/generated/Rules';
import { EMPTY_RULES, MAIL, SAMPLE_RULES, SLACK, SLACK_TARGET, YOUTUBE } from '@/test/ipc';

const EVERYTHING_LOCKED: RulesLocks = { blocking: true, limits: true };

function describeLoosening(saved: Rules, draft: Rules, locks: RulesLocks = EVERYTHING_LOCKED) {
  return describeLocked(saved, draft, locks);
}

function withSites(...sites: string[]): Rules {
  return { ...EMPTY_RULES, blockedSites: sites };
}

describe('what counts as a loosening', () => {
  it('finds nothing when the draft is the same or tighter', () => {
    expect(describeLoosening(SAMPLE_RULES, SAMPLE_RULES)).toBeNull();
    const tighter = addApp(addSite(addWindow(SAMPLE_RULES), 'news.com'), MAIL);
    expect(describeLoosening(SAMPLE_RULES, setLimitMinutes(tighter, YOUTUBE, 15))).toBeNull();
  });

  it('finds a removed site, app and limit', () => {
    const draft = removeLimit(
      removeApp(removeSite(SAMPLE_RULES, 'reddit.com'), SLACK.bundleId),
      YOUTUBE,
    );
    expect(describeLoosening(SAMPLE_RULES, draft)).toBe(
      'Remove reddit.com, stop blocking Slack and remove the limit for youtube.com.',
    );
  });

  it('finds a raised limit but not a lowered one', () => {
    expect(describeLoosening(SAMPLE_RULES, setLimitMinutes(SAMPLE_RULES, YOUTUBE, 60))).toBe(
      'Raise the limit for youtube.com from 30 minutes to 1 hour.',
    );
    expect(describeLoosening(SAMPLE_RULES, setLimitMinutes(SAMPLE_RULES, YOUTUBE, 10))).toBeNull();
  });

  it('finds a shortened schedule in minutes per week', () => {
    const draft = updateWindow(SAMPLE_RULES, 0, { end: 960 });
    expect(describeLoosening(SAMPLE_RULES, draft)).toBe('Cut 5 hours of focus time each week.');
  });

  it('does not count a site whose host is still blocked by a kept site', () => {
    const saved = withSites('m.youtube.com', 'youtube.com');
    expect(describeLoosening(saved, withSites('youtube.com'))).toBeNull();
    expect(describeLoosening(saved, withSites('m.youtube.com'))).toBe('Remove youtube.com.');
  });

  it.each(['youtube.com', 'www.youtube.com', 'm.youtube.com'])(
    'keeps %s blocked while youtube.com stays on the list, as the hosts file does',
    (removed) => {
      expect(describeLoosening(withSites(removed), withSites('youtube.com'))).toBeNull();
    },
  );

  it.each([
    'studio.youtube.com',
    'a.b.youtube.com',
    'music.youtube.com',
    'old.youtube.com',
    'mobile.youtube.com',
  ])(
    'counts %s as unblocked when only youtube.com stays, because the hosts file has no wildcards',
    (removed) => {
      expect(describeLoosening(withSites(removed), withSites('youtube.com'))).toBe(
        `Remove ${removed}.`,
      );
    },
  );

  it('asks for an unlock before swapping a subdomain for its parent', () => {
    const saved = withSites('studio.youtube.com');
    expect(describeLoosening(saved, withSites('youtube.com'))).toBe('Remove studio.youtube.com.');
  });

  it('does not treat a lookalike domain as a parent', () => {
    expect(describeLoosening(withSites('youtube.com'), withSites('notyoutube.com'))).toBe(
      'Remove youtube.com.',
    );
  });

  it('treats an app limit as the same limit when only its name changed', () => {
    const saved = addLimit(EMPTY_RULES, SLACK_TARGET, 30);
    const renamed = addLimit(EMPTY_RULES, { kind: 'app', app: { ...SLACK, name: 'Slack 2' } }, 30);
    expect(describeLoosening(saved, renamed)).toBeNull();
  });
});

describe('describeLoosening', () => {
  it('is null when nothing is loosened', () => {
    expect(describeLoosening(SAMPLE_RULES, SAMPLE_RULES)).toBeNull();
    expect(describeLoosening(SAMPLE_RULES, addSite(SAMPLE_RULES, 'x.com'))).toBeNull();
  });

  it('names one removed site', () => {
    expect(describeLoosening(SAMPLE_RULES, removeSite(SAMPLE_RULES, 'reddit.com'))).toBe(
      'Remove reddit.com.',
    );
  });

  it('names two removed sites and counts beyond that', () => {
    const saved = withSites('a.com', 'b.com', 'c.com', 'd.com');
    expect(describeLoosening(saved, withSites('c.com', 'd.com'))).toBe('Remove a.com and b.com.');
    expect(describeLoosening(saved, withSites('d.com'))).toBe('Remove a.com and 2 other sites.');
    expect(describeLoosening(withSites('a.com', 'b.com', 'c.com'), EMPTY_RULES)).toBe(
      'Remove a.com and 2 other sites.',
    );
  });

  it('names removed apps', () => {
    expect(describeLoosening(SAMPLE_RULES, removeApp(SAMPLE_RULES, SLACK.bundleId))).toBe(
      'Stop blocking Slack.',
    );
    const two: Rules = { ...EMPTY_RULES, blockedApps: [MAIL, SLACK] };
    expect(describeLoosening(two, EMPTY_RULES)).toBe('Stop blocking Mail and Slack.');
    const three: Rules = {
      ...two,
      blockedApps: [...two.blockedApps, { bundleId: 'x.y', name: 'Xy' }],
    };
    expect(describeLoosening(three, EMPTY_RULES)).toBe('Stop blocking Mail and 2 other apps.');
  });

  it('describes removed and raised limits by name', () => {
    expect(describeLoosening(SAMPLE_RULES, removeLimit(SAMPLE_RULES, YOUTUBE))).toBe(
      'Remove the limit for youtube.com.',
    );
    expect(describeLoosening(SAMPLE_RULES, setLimitMinutes(SAMPLE_RULES, YOUTUBE, 90))).toBe(
      'Raise the limit for youtube.com from 30 minutes to 1 hour 30 minutes.',
    );
  });

  it('groups several limits', () => {
    const base = addLimit(
      addLimit(SAMPLE_RULES, SLACK_TARGET, 20),
      { kind: 'site', domain: 'x.com' },
      15,
    );
    const raisedAll = [YOUTUBE, SLACK_TARGET, { kind: 'site' as const, domain: 'x.com' }].reduce(
      (rules, target) => setLimitMinutes(rules, target, 120),
      base,
    );
    expect(
      describeLoosening(
        base,
        setLimitMinutes(setLimitMinutes(base, YOUTUBE, 60), SLACK_TARGET, 60),
      ),
    ).toBe('Raise the limits for youtube.com and Slack.');
    expect(describeLoosening(base, raisedAll)).toBe('Raise 3 daily limits.');
    expect(describeLoosening(base, { ...base, limits: [] })).toBe('Remove 3 daily limits.');
    expect(describeLoosening(base, { ...base, limits: base.limits.slice(2) })).toBe(
      'Remove the limits for youtube.com and Slack.',
    );
  });

  it('describes a shorter schedule in hours and minutes', () => {
    expect(describeLoosening(SAMPLE_RULES, updateWindow(SAMPLE_RULES, 0, { end: 990 }))).toBe(
      'Cut 2 hours 30 minutes of focus time each week.',
    );
    expect(describeLoosening(SAMPLE_RULES, { ...SAMPLE_RULES, schedule: { windows: [] } })).toBe(
      'Cut 40 hours of focus time each week.',
    );
  });

  it('joins several changes into one sentence in a stable order', () => {
    const draft = updateWindow(
      setLimitMinutes(
        removeApp(removeSite(SAMPLE_RULES, 'reddit.com'), SLACK.bundleId),
        YOUTUBE,
        45,
      ),
      0,
      { end: 1000 },
    );
    expect(describeLoosening(SAMPLE_RULES, draft)).toBe(
      'Remove reddit.com, stop blocking Slack, raise the limit for youtube.com from 30 minutes to 45 minutes and cut 1 hour 40 minutes of focus time each week.',
    );
  });
});

describe('what is locked right now', () => {
  const raised = setLimitMinutes(SAMPLE_RULES, YOUTUBE, 60);
  const removed = removeLimit(SAMPLE_RULES, YOUTUBE);

  it('locks limits outside focus time, where nothing else is locked', () => {
    const outsideFocus: RulesLocks = { blocking: false, limits: true };

    expect(describeLoosening(SAMPLE_RULES, raised, outsideFocus)).toBe(
      'Raise the limit for youtube.com from 30 minutes to 1 hour.',
    );
    expect(describeLoosening(SAMPLE_RULES, removed, outsideFocus)).toBe(
      'Remove the limit for youtube.com.',
    );
    expect(
      describeLoosening(SAMPLE_RULES, removeSite(SAMPLE_RULES, 'reddit.com'), outsideFocus),
    ).toBeNull();
    expect(
      describeLoosening(SAMPLE_RULES, updateWindow(SAMPLE_RULES, 0, { end: 960 }), outsideFocus),
    ).toBeNull();
  });

  it('names only the limits when a mixed change is saved outside focus time', () => {
    const mixed = removeSite(raised, 'reddit.com');

    expect(describeLoosening(SAMPLE_RULES, mixed, { blocking: false, limits: true })).toBe(
      'Raise the limit for youtube.com from 30 minutes to 1 hour.',
    );
  });

  it('locks nothing once an unlock is active', () => {
    const unlocked: RulesLocks = { blocking: false, limits: false };

    expect(describeLoosening(SAMPLE_RULES, raised, unlocked)).toBeNull();
    expect(describeLoosening(SAMPLE_RULES, removed, unlocked)).toBeNull();
  });
});

describe('held sites', () => {
  const held = ['twitch.tv', 'reddit.com'];

  it('counts a held site the draft does not block as a removal', () => {
    expect(describeLocked(SAMPLE_RULES, SAMPLE_RULES, EVERYTHING_LOCKED, held)).toBe(
      'Remove twitch.tv.',
    );
  });

  it('is satisfied by a draft site that blocks the held one, aliases included', () => {
    const draft = withSites('twitch.tv', 'www.reddit.com');

    expect(describeLocked(draft, draft, EVERYTHING_LOCKED, held)).toBe('Remove reddit.com.');
    expect(
      describeLocked(draft, withSites('twitch.tv', 'reddit.com'), EVERYTHING_LOCKED, held),
    ).toBeNull();
  });

  it('is not locked once an unlock is active', () => {
    const unlocked: RulesLocks = { blocking: false, limits: false };

    expect(describeLocked(EMPTY_RULES, EMPTY_RULES, unlocked, held)).toBeNull();
  });
});

describe('includesLockedLimitChange', () => {
  const whileBlocking: RulesLocks = { blocking: true, limits: true };
  const limitRaise = setLimitMinutes(SAMPLE_RULES, YOUTUBE, 60);
  const siteRemoval = removeSite(SAMPLE_RULES, 'reddit.com');
  const mixed = removeSite(limitRaise, 'reddit.com');

  it('is true when only a limit is raised or removed', () => {
    expect(includesLockedLimitChange(SAMPLE_RULES, limitRaise, whileBlocking)).toBe(true);
    expect(
      includesLockedLimitChange(SAMPLE_RULES, removeLimit(SAMPLE_RULES, YOUTUBE), whileBlocking),
    ).toBe(true);
  });

  it('is false when only sites, apps or the schedule are loosened', () => {
    const shorter = updateWindow(SAMPLE_RULES, 0, { end: 960 });
    expect(includesLockedLimitChange(SAMPLE_RULES, siteRemoval, whileBlocking)).toBe(false);
    expect(includesLockedLimitChange(SAMPLE_RULES, shorter, whileBlocking)).toBe(false);
  });

  it('is true for a mix of a limit change and another loosening', () => {
    expect(includesLockedLimitChange(SAMPLE_RULES, mixed, whileBlocking)).toBe(true);
  });

  it('is false when nothing is locked or the limit is only lowered', () => {
    const unlocked: RulesLocks = { blocking: false, limits: false };
    expect(includesLockedLimitChange(SAMPLE_RULES, limitRaise, unlocked)).toBe(false);
    expect(
      includesLockedLimitChange(
        SAMPLE_RULES,
        setLimitMinutes(SAMPLE_RULES, YOUTUBE, 10),
        whileBlocking,
      ),
    ).toBe(false);
  });
});
