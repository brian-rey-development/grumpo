import { render, screen } from '@testing-library/react';
import { expect, it } from 'vitest';

import { LooseningNotice } from '@/features/rules/RulesNotices';

const WAIT_BODY =
  "These changes weaken focus while it's on. Unlock to save them, or save them when the block ends.";
const LIMITS_BODY =
  'Raising or removing a daily limit always needs an unlock, even outside focus time. Unlock to save these changes.';

function renderNotice(blocking: boolean, includesLimitChange: boolean) {
  render(
    <LooseningNotice
      onUnlock={() => undefined}
      blocking={blocking}
      includesLimitChange={includesLimitChange}
    />,
  );
}

it('says a limit raise needs an unlock even during focus time', () => {
  renderNotice(true, true);

  expect(screen.getByText(LIMITS_BODY)).toBeTruthy();
  expect(screen.queryByText(WAIT_BODY)).toBeNull();
});

it('offers waiting for the block to end when only sites, apps or the schedule are locked', () => {
  renderNotice(true, false);

  expect(screen.getByText(WAIT_BODY)).toBeTruthy();
  expect(screen.queryByText(LIMITS_BODY)).toBeNull();
});

it('leads with the limits wording when a limit change is mixed with other loosenings', () => {
  renderNotice(true, true);

  expect(screen.getByText(LIMITS_BODY)).toBeTruthy();
});

it('keeps the limits wording outside focus time', () => {
  renderNotice(false, false);

  expect(screen.getByText(LIMITS_BODY)).toBeTruthy();
});
