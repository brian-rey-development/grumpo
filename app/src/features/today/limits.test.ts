import { describe, expect, it } from 'vitest';

import { limitUsage } from '@/test/ipc';

import { describeLimit } from './limits';

const MINUTE = 60;
const NEAR_LIMIT_SECONDS = 5 * MINUTE;

describe('describeLimit', () => {
  it('is fine while there is plenty of time left', () => {
    const progress = describeLimit(
      limitUsage({ usedSeconds: 38 * MINUTE, limitSeconds: 60 * MINUTE }),
    );

    expect(progress.level).toBe('ok');
    expect(progress.fraction).toBeCloseTo(38 / 60);
    expect(progress.remainingSeconds).toBe(22 * MINUTE);
  });

  it('turns near exactly at the five minute mark', () => {
    const limitSeconds = 60 * MINUTE;

    expect(
      describeLimit(
        limitUsage({ usedSeconds: limitSeconds - NEAR_LIMIT_SECONDS - 1, limitSeconds }),
      ).level,
    ).toBe('ok');
    expect(
      describeLimit(limitUsage({ usedSeconds: limitSeconds - NEAR_LIMIT_SECONDS, limitSeconds }))
        .level,
    ).toBe('near');
  });

  it('is reached whenever the service says so and never overfills the bar', () => {
    const atLimit = describeLimit(
      limitUsage({ usedSeconds: 60 * MINUTE, limitSeconds: 60 * MINUTE, reached: true }),
    );
    const beyond = describeLimit(
      limitUsage({ usedSeconds: 75 * MINUTE, limitSeconds: 60 * MINUTE, reached: true }),
    );
    const early = describeLimit(
      limitUsage({ usedSeconds: 10 * MINUTE, limitSeconds: 60 * MINUTE, reached: true }),
    );

    expect(atLimit.level).toBe('reached');
    expect(beyond.level).toBe('reached');
    expect(early.level).toBe('reached');
    expect(beyond.fraction).toBe(1);
    expect(beyond.remainingSeconds).toBe(0);
  });

  it('is not reached while the service says it is not, even with no minutes left', () => {
    const level = describeLimit(
      limitUsage({ usedSeconds: 60 * MINUTE, limitSeconds: 60 * MINUTE, reached: false }),
    ).level;

    expect(level).toBe('near');
  });

  it('fills the bar for a zero limit', () => {
    expect(describeLimit(limitUsage({ usedSeconds: 0, limitSeconds: 0 })).fraction).toBe(1);
  });
});
