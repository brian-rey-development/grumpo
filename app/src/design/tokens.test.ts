import { describe, expect, it } from 'vitest';

import { contrastRatio, hueDegrees, type Rgb } from '@/test/contrast';
import { colorOf, opaque, type Scheme, type TokenSet, tokensFor } from '@/test/tokens';

const TEXT_RATIO = 4.5;
const GRAPHIC_RATIO = 3;
const SCHEMES: readonly Scheme[] = ['light', 'dark'];
const STATES = ['focus', 'unlocked', 'limit'] as const;
const MIN_HUE_DISTANCE = 60;
const FULL_TURN = 360;
const BRAND = {
  orange: '#ff8a3d',
  cream: '#fff4e6',
  charcoal: '#1f1f23',
  red: '#ff4d4d',
} as const;
const BANNERS = [
  ['on duty', '--color-accent', '--color-on-accent'],
  ['limit reached', '--color-limit', '--color-on-limit'],
  ['unlocked', '--color-unlocked', '--color-on-unlocked'],
  ['service missing', '--color-charcoal', '--color-cream'],
] as const;

interface Surfaces {
  tokens: TokenSet;
  surface: Rgb;
  raised: Rgb;
  grouped: Rgb;
  track: Rgb;
}

function surfacesFor(scheme: Scheme): Surfaces {
  const tokens = tokensFor(scheme);
  const surface = colorOf(tokens, '--color-surface').rgb;
  const raised = colorOf(tokens, '--color-surface-raised').rgb;
  return {
    tokens,
    surface,
    raised,
    grouped: opaque(tokens, '--color-fill-grouped', surface),
    track: colorOf(tokens, '--color-track').rgb,
  };
}

function solid(tokens: TokenSet, name: string): Rgb {
  return colorOf(tokens, name).rgb;
}

function hueGap(first: Rgb, second: Rgb): number {
  const gap = Math.abs(hueDegrees(first) - hueDegrees(second));
  return Math.min(gap, FULL_TURN - gap);
}

describe('brand palette', () => {
  it('keeps the owner colors exactly where the brand puts them', () => {
    const light = tokensFor('light');
    const dark = tokensFor('dark');
    expect(light['--color-accent']).toBe(BRAND.orange);
    expect(dark['--color-accent']).toBe(BRAND.orange);
    expect(light['--color-limit']).toBe(BRAND.red);
    expect(dark['--color-limit']).toBe(BRAND.red);
    expect(light['--color-charcoal']).toBe(BRAND.charcoal);
    expect(light['--color-cream']).toBe(BRAND.cream);
    expect(light['--color-ink']).toBe(BRAND.charcoal);
    expect(dark['--color-ink']).toBe(BRAND.cream);
  });

  it.each(SCHEMES)(
    'keeps unlocked clearly apart from the orange and red states in %s',
    (scheme) => {
      const tokens = tokensFor(scheme);
      const unlocked = solid(tokens, '--color-unlocked');
      for (const name of ['--color-focus', '--color-limit']) {
        expect(hueGap(unlocked, solid(tokens, name)), name).toBeGreaterThanOrEqual(
          MIN_HUE_DISTANCE,
        );
      }
    },
  );
});

describe.each(SCHEMES)('%s palette text contrast', (scheme) => {
  const { tokens, surface, raised, grouped, track } = surfacesFor(scheme);

  it('keeps primary and secondary ink readable on every surface', () => {
    for (const background of [surface, raised, track]) {
      expect(contrastRatio(solid(tokens, '--color-ink'), background)).toBeGreaterThanOrEqual(
        TEXT_RATIO,
      );
    }
    for (const background of [surface, raised, grouped, track]) {
      expect(
        contrastRatio(solid(tokens, '--color-ink-secondary'), background),
      ).toBeGreaterThanOrEqual(TEXT_RATIO);
    }
  });

  it('keeps accent text readable on every surface', () => {
    const accentText = solid(tokens, '--color-accent-strong');
    for (const background of [surface, raised, grouped]) {
      expect(contrastRatio(accentText, background)).toBeGreaterThanOrEqual(TEXT_RATIO);
    }
  });

  it.each(BANNERS)('puts readable text on the %s banner', (_name, fill, text) => {
    expect(contrastRatio(solid(tokens, text), solid(tokens, fill))).toBeGreaterThanOrEqual(
      TEXT_RATIO,
    );
  });

  it('puts readable text on the focus fill and the tooltip', () => {
    expect(
      contrastRatio(solid(tokens, '--color-on-accent'), solid(tokens, '--color-focus')),
    ).toBeGreaterThanOrEqual(TEXT_RATIO);
    expect(
      contrastRatio(solid(tokens, '--color-on-tooltip'), solid(tokens, '--color-tooltip')),
    ).toBeGreaterThanOrEqual(TEXT_RATIO);
  });

  it.each(STATES)('keeps %s status text readable on its tint', (state) => {
    const text = solid(tokens, `--color-${state}-text`);
    for (const background of [surface, raised]) {
      const tint = opaque(tokens, `--color-${state}-tint`, background);
      expect(contrastRatio(text, tint)).toBeGreaterThanOrEqual(TEXT_RATIO);
    }
  });

  it('keeps the sidebar text readable on the charcoal spine', () => {
    const spine = solid(tokens, '--color-charcoal');
    expect(contrastRatio(solid(tokens, '--color-cream'), spine)).toBeGreaterThanOrEqual(TEXT_RATIO);
    expect(contrastRatio(solid(tokens, '--color-on-sidebar-dim'), spine)).toBeGreaterThanOrEqual(
      TEXT_RATIO,
    );
    const hover = opaque(tokens, '--color-sidebar-hover', spine);
    expect(contrastRatio(solid(tokens, '--color-cream'), hover)).toBeGreaterThanOrEqual(TEXT_RATIO);
    expect(
      contrastRatio(solid(tokens, '--color-charcoal'), solid(tokens, '--color-cream')),
    ).toBeGreaterThanOrEqual(TEXT_RATIO);
  });
});

describe.each(SCHEMES)('%s palette graphics contrast', (scheme) => {
  const { tokens, surface, raised, grouped, track } = surfacesFor(scheme);

  it('draws the focus ring against every surface', () => {
    const ring = solid(tokens, '--color-ring');
    expect(ring).toEqual(solid(tokens, '--color-accent-strong'));
    expect(colorOf(tokens, '--color-ring').alpha).toBe(1);
    for (const background of [surface, raised, grouped]) {
      expect(contrastRatio(ring, background)).toBeGreaterThanOrEqual(GRAPHIC_RATIO);
    }
  });

  it('separates the field border and the toggle outline from neighbours', () => {
    const control = solid(tokens, '--color-control');
    for (const background of [surface, raised, grouped]) {
      expect(contrastRatio(control, background)).toBeGreaterThanOrEqual(GRAPHIC_RATIO);
      expect(contrastRatio(solid(tokens, '--color-ink'), background)).toBeGreaterThanOrEqual(
        GRAPHIC_RATIO,
      );
    }
  });

  it('keeps the toggle thumb readable on the off track and the orange track', () => {
    const thumb = solid(tokens, '--color-on-accent');
    expect(contrastRatio(thumb, solid(tokens, '--color-control'))).toBeGreaterThanOrEqual(
      GRAPHIC_RATIO,
    );
    expect(contrastRatio(thumb, solid(tokens, '--color-accent'))).toBeGreaterThanOrEqual(
      GRAPHIC_RATIO,
    );
  });

  it('keeps chart bars and the focus data color visible on panels and the page', () => {
    for (const background of [surface, raised]) {
      expect(contrastRatio(solid(tokens, '--color-bar'), background)).toBeGreaterThanOrEqual(
        GRAPHIC_RATIO,
      );
      expect(contrastRatio(solid(tokens, '--color-focus'), background)).toBeGreaterThanOrEqual(
        GRAPHIC_RATIO,
      );
    }
    expect(contrastRatio(solid(tokens, '--color-bar'), track)).toBeGreaterThanOrEqual(
      GRAPHIC_RATIO,
    );
  });

  it('tells a track from the panel it sits on', () => {
    expect(contrastRatio(track, raised)).toBeGreaterThan(1.05);
  });

  it.each(STATES)('keeps the %s indicator visible', (state) => {
    for (const background of [surface, raised]) {
      expect(contrastRatio(solid(tokens, `--color-${state}`), background)).toBeGreaterThanOrEqual(
        GRAPHIC_RATIO,
      );
    }
  });
});
