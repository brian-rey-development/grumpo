import { describe, expect, it } from 'vitest';

import { duration } from '@/design/motion';
import { tokensFor } from '@/test/tokens';

const MS_PER_SECOND = 1000;

describe('motion timings', () => {
  const tokens = tokensFor('light');

  it.each(Object.entries(duration))('mirrors the %s duration in tokens.css', (name, seconds) => {
    expect(tokens[`--duration-${name}`]).toBe(
      `${Math.round(seconds * MS_PER_SECOND).toString()}ms`,
    );
  });

  it('mirrors the easing curve in tokens.css', () => {
    expect(tokens['--ease-out']).toBe('cubic-bezier(0.2, 0.8, 0.2, 1)');
  });
});
