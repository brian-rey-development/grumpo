/// <reference types="node" />
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';

import { CountdownRing } from '@/features/unlock/CountdownRing';
import { mockMatchMedia } from '@/test/matchMedia';

const CIRCUMFERENCE = 2 * Math.PI * 46;
const STYLES = resolve(process.cwd(), 'src/features/unlock/CountdownRing.module.css');

function arcOffset(): number {
  const arc = screen.getByTestId('countdown-arc').querySelector('circle:last-of-type');
  return Number(arc?.getAttribute('stroke-dashoffset'));
}

describe.each([
  ['with full motion', false],
  ['with reduced motion', true],
])('CountdownRing %s', (_, reducedMotion) => {
  beforeEach(() => {
    mockMatchMedia({ reducedMotion });
  });

  it('counts down as m:ss, the way the sidebar pill does', () => {
    render(<CountdownRing remainingSeconds={291} totalSeconds={600} />);

    expect(screen.getByRole('timer').textContent).toBe('4:51');
  });

  it('keeps the arc, sized by the time left out of the whole wait', () => {
    render(<CountdownRing remainingSeconds={150} totalSeconds={600} />);

    expect(arcOffset()).toBeCloseTo(CIRCUMFERENCE * 0.75, 5);
  });

  it('shows a full arc at the start and an empty one at the end', () => {
    const { rerender } = render(<CountdownRing remainingSeconds={300} totalSeconds={300} />);
    expect(arcOffset()).toBeCloseTo(0, 5);

    rerender(<CountdownRing remainingSeconds={0} totalSeconds={300} />);
    expect(arcOffset()).toBeCloseTo(CIRCUMFERENCE, 5);
  });

  it('never draws more than a full arc or divides by a missing total', () => {
    const { rerender } = render(<CountdownRing remainingSeconds={900} totalSeconds={300} />);
    expect(arcOffset()).toBeCloseTo(0, 5);

    rerender(<CountdownRing remainingSeconds={30} totalSeconds={0} />);
    expect(arcOffset()).toBeCloseTo(CIRCUMFERENCE, 5);
  });
});

describe('CountdownRing colour', () => {
  it('draws the arc in the amber of an unlock in progress, like the pill', () => {
    const css = readFileSync(STYLES, 'utf8');

    expect(css).toMatch(/\.arc\s*\{[^}]*stroke:\s*var\(--color-unlocked\)/);
  });
});

it.each([
  [291, '4 minutes 51 seconds'],
  [61, '1 minute 1 second'],
  [3_600, '1 hour'],
  [3_725, '1 hour 2 minutes 5 seconds'],
  [0, '0 seconds'],
])('names the time left for screen readers, saying %i seconds as %s', (seconds, spoken) => {
  render(<CountdownRing remainingSeconds={seconds} totalSeconds={3_725} />);

  expect(screen.getByRole('timer').getAttribute('aria-label')).toContain(spoken);
});
