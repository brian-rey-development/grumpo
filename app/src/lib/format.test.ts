import { describe, expect, it } from 'vitest';

import {
  formatClock,
  formatClockFromSeconds,
  formatClockRange,
  formatCountdown,
  formatDayLabel,
  formatDuration,
  formatFullDate,
  formatHourLabel,
  formatMonthDay,
  formatDurationLong,
  formatTimeOfDay,
  formatTimeRange,
  pluralize,
  splitDuration,
} from '@/lib/format';

describe('formatDuration', () => {
  it.each([
    [15_120, '4h 12min'],
    [2_280, '38min'],
    [3_600, '1h'],
    [3_660, '1h 1min'],
    [60, '1min'],
    [59, '< 1min'],
    [1, '< 1min'],
    [0, '0min'],
  ])('formats %i seconds as %s', (seconds, expected) => {
    expect(formatDuration(seconds)).toBe(expected);
  });

  it('treats negative and non-finite values as zero', () => {
    expect(formatDuration(-30)).toBe('0min');
    expect(formatDuration(Number.NaN)).toBe('0min');
    expect(formatDuration(Number.POSITIVE_INFINITY)).toBe('0min');
  });

  it('floors partial minutes so the total never overstates', () => {
    expect(formatDuration(3_599)).toBe('59min');
  });
});

describe('splitDuration', () => {
  it('separates hours and minutes', () => {
    expect(splitDuration(15_120)).toEqual({
      hours: 4,
      minutes: 12,
      seconds: 0,
      underMinute: false,
    });
    expect(splitDuration(3_725).seconds).toBe(5);
  });

  it('flags durations below a minute', () => {
    expect(splitDuration(20).underMinute).toBe(true);
    expect(splitDuration(0).underMinute).toBe(false);
  });
});

describe('formatClock', () => {
  it('uses a zero padded 24 hour clock in 24 hour locales', () => {
    expect(formatClock(new Date(2026, 9, 12, 19, 0), { locale: 'en-GB' })).toBe('19:00');
    expect(formatClock(new Date(2026, 9, 12, 7, 5), { locale: 'en-GB' })).toBe('07:05');
    expect(formatClock(new Date(2026, 9, 12, 0, 0), { locale: 'en-GB' })).toBe('00:00');
  });

  it('uses a 12 hour clock in 12 hour locales', () => {
    expect(formatClock(new Date(2026, 9, 12, 19, 0), { locale: 'en-US' })).toMatch(/^7:00\sPM$/);
    expect(formatClock(new Date(2026, 9, 12, 0, 5), { locale: 'en-US' })).toMatch(/^12:05\sAM$/);
  });

  it('follows the system locale when none is given', () => {
    const date = new Date(2026, 9, 12, 19, 0);

    expect(formatClock(date)).toBe(formatClock(date, { locale: 'en-GB' }));
  });
});

describe('formatClockFromSeconds', () => {
  it('converts seconds of day to a clock', () => {
    expect(formatClockFromSeconds(0, { locale: 'en-GB' })).toBe('00:00');
    expect(formatClockFromSeconds(68_400, { locale: 'en-GB' })).toBe('19:00');
    expect(formatClockFromSeconds(86_400, { locale: 'en-GB' })).toBe('00:00');
    expect(formatClockFromSeconds(68_400, { locale: 'en-US' })).toMatch(/^7:00\sPM$/);
  });
});

describe('formatCountdown', () => {
  it('shows minutes and seconds, adding hours when needed', () => {
    expect(formatCountdown(754)).toBe('12:34');
    expect(formatCountdown(0)).toBe('00:00');
    expect(formatCountdown(3_725)).toBe('1:02:05');
  });

  it('rounds partial seconds up so the countdown never shows 00:00 early', () => {
    expect(formatCountdown(0.2)).toBe('00:01');
  });

  it('can leave the leading minutes unpadded', () => {
    expect(formatCountdown(252, { padMinutes: false })).toBe('4:12');
    expect(formatCountdown(3_725, { padMinutes: false })).toBe('1:02:05');
  });
});

describe('formatMonthDay and formatFullDate', () => {
  const date = new Date(2026, 9, 5);

  it('formats a short day and month without the weekday', () => {
    expect(formatMonthDay(date, 'en-GB')).toBe('5 Oct');
    expect(formatMonthDay(date, 'en-US')).toBe('Oct 5');
  });

  it('formats the whole date for headings', () => {
    expect(formatFullDate(date, 'en-GB')).toMatch(/^Monday,? 5 October 2026$/);
    expect(formatFullDate(date, 'en-US')).toBe('Monday, October 5, 2026');
  });
});

describe('formatDayLabel', () => {
  const today = new Date(2026, 9, 12, 15, 30);

  it('labels today and yesterday', () => {
    expect(formatDayLabel(new Date(2026, 9, 12, 0, 5), today)).toBe('Today');
    expect(formatDayLabel(new Date(2026, 9, 11, 23, 59), today)).toBe('Yesterday');
  });

  it('labels older days in the locale of the system', () => {
    expect(formatDayLabel(new Date(2026, 9, 5), today, 'en-GB')).toBe('Mon 5 Oct');
    expect(formatDayLabel(new Date(2026, 9, 7), today, 'en-GB')).toBe('Wed 7 Oct');
    expect(formatDayLabel(new Date(2026, 9, 5), today, 'en-US')).toBe('Mon, Oct 5');
  });

  it('stays correct across a daylight saving boundary', () => {
    const afterClockChange = new Date(2026, 10, 2, 12);
    expect(formatDayLabel(new Date(2026, 10, 1, 12), afterClockChange)).toBe('Yesterday');
  });
});

describe('formatHourLabel', () => {
  it('is a bare padded hour on a 24 hour clock', () => {
    expect([0, 6, 12, 18].map((hour) => formatHourLabel(hour, { locale: 'en-GB' }))).toEqual([
      '00',
      '06',
      '12',
      '18',
    ]);
  });

  it('is a short hour with the day period on a 12 hour clock', () => {
    expect(
      [0, 6, 12, 18].map((hour) => formatHourLabel(hour, { locale: 'en-US' }).replace(/\s/g, ' ')),
    ).toEqual(['12 AM', '6 AM', '12 PM', '6 PM']);
  });

  it('wraps 24 round to midnight so the last tick of a day reads like the first', () => {
    expect(formatHourLabel(24, { locale: 'en-US' })).toBe(formatHourLabel(0, { locale: 'en-US' }));
    expect(formatHourLabel(-1, { locale: 'en-US' })).toBe(formatHourLabel(23, { locale: 'en-US' }));
  });
});

describe('formatDurationLong', () => {
  it.each([
    [300, '5 minutes'],
    [60, '1 minute'],
    [3_600, '1 hour'],
    [5_400, '1 hour 30 minutes'],
    [7_320, '2 hours 2 minutes'],
    [252, '4 minutes 12 seconds'],
    [1, '1 second'],
    [0, '0 seconds'],
  ])('spells %i seconds out as %s', (seconds, expected) => {
    expect(formatDurationLong(seconds)).toBe(expected);
  });
});

describe('pluralize', () => {
  it('uses the singular only for exactly one', () => {
    expect(pluralize(1, 'day')).toBe('1 day');
    expect(pluralize(0, 'day')).toBe('0 days');
    expect(pluralize(30, 'day')).toBe('30 days');
  });
});

describe('formatTimeRange', () => {
  it('joins two times with "to"', () => {
    expect(formatTimeRange('09:00', '17:00')).toBe('09:00 to 17:00');
  });
});

describe('formatClockRange', () => {
  it('formats a stretch of the day from seconds', () => {
    expect(formatClockRange(32_400, 36_000, { locale: 'en-GB' })).toBe('09:00 to 10:00');
  });
});

describe('formatTimeOfDay', () => {
  it('formats minutes since midnight', () => {
    expect(formatTimeOfDay(540, undefined)).toBe('09:00');
    expect(formatTimeOfDay(0)).toBe('00:00');
  });

  it('uses the given wording for the very end of the day', () => {
    expect(formatTimeOfDay(1_440, 'End of day')).toBe('End of day');
    expect(formatTimeOfDay(1_439, 'End of day')).toBe('23:59');
  });
});

describe('the clock style', () => {
  const evening = new Date(2026, 9, 12, 19, 5);
  const morning = new Date(2026, 9, 12, 7, 5);

  it('forces a 12 hour clock even where the locale uses 24 hours', () => {
    expect(formatClock(evening, { clockStyle: 'h12', locale: 'en-GB' })).toMatch(/^7:05\spm$/i);
  });

  it('forces a padded 24 hour clock even where the locale uses 12 hours', () => {
    expect(formatClock(evening, { clockStyle: 'h23', locale: 'en-US' })).toBe('19:05');
    expect(formatClock(morning, { clockStyle: 'h23', locale: 'en-US' })).toBe('07:05');
  });

  it('follows the locale until a style is known', () => {
    expect(formatClock(evening, { locale: 'en-GB' })).toBe('19:05');
    expect(formatClock(evening, { clockStyle: undefined, locale: 'en-US' })).toMatch(/PM$/);
  });

  it('carries the style to hour labels, ranges and times of day', () => {
    const h12 = { clockStyle: 'h12', locale: 'en-GB' } as const;
    const h23 = { clockStyle: 'h23', locale: 'en-US' } as const;

    expect(formatHourLabel(18, h12).replace(/\s/g, ' ')).toMatch(/^6 pm$/i);
    expect(formatHourLabel(6, h23)).toBe('06');
    expect(formatClockRange(32_400, 36_000, h23)).toBe('09:00 to 10:00');
    expect(formatTimeOfDay(1_020, 'End of day', h23)).toBe('17:00');
    expect(formatTimeOfDay(1_020, 'End of day', h12).replace(/\s/g, ' ')).toMatch(/^5:00 pm$/i);
  });
});
