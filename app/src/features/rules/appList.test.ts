import { describe, expect, it } from 'vitest';

import { filterApps, optionId } from '@/features/rules/appList';
import { KNOWN_APPS, MAIL, NOTES, SAFARI, SLACK } from '@/test/ipc';

const NONE: ReadonlySet<string> = new Set();

describe('filterApps', () => {
  it('lists running apps first and then the rest by name', () => {
    expect(filterApps(KNOWN_APPS, NONE, '').map(({ app }) => app.name)).toEqual([
      'Safari',
      'Mail',
      'Notes',
      'Slack',
    ]);
  });

  it('leaves out excluded apps', () => {
    const excluded = new Set([SAFARI.bundleId, MAIL.bundleId]);
    expect(filterApps(KNOWN_APPS, excluded, '').map(({ app }) => app.name)).toEqual([
      'Notes',
      'Slack',
    ]);
  });

  it('matches names without regard to case or surrounding space', () => {
    expect(filterApps(KNOWN_APPS, NONE, '  sLaC ').map(({ app }) => app)).toEqual([SLACK]);
  });

  it('matches bundle ids too', () => {
    expect(filterApps(KNOWN_APPS, NONE, 'apple.notes').map(({ app }) => app)).toEqual([NOTES]);
  });

  it('does not change the list it was given', () => {
    const copy = [...KNOWN_APPS];
    filterApps(KNOWN_APPS, NONE, '');
    expect(KNOWN_APPS).toEqual(copy);
  });
});

it('builds an option id from the list id and the bundle id', () => {
  expect(optionId('list', SLACK)).toBe(`list-${SLACK.bundleId}`);
});
