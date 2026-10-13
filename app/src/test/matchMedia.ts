import { vi } from 'vitest';

interface MediaPreferences {
  reducedMotion?: boolean;
  dark?: boolean;
}

export function mockMatchMedia({
  reducedMotion = false,
  dark = false,
}: MediaPreferences = {}): void {
  vi.stubGlobal('matchMedia', (query: string): MediaQueryList => ({
    matches:
      (query.includes('prefers-reduced-motion') && reducedMotion) ||
      (query.includes('prefers-color-scheme: dark') && dark),
    media: query,
    onchange: null,
    addEventListener: () => undefined,
    removeEventListener: () => undefined,
    addListener: () => undefined,
    removeListener: () => undefined,
    dispatchEvent: () => false,
  }));
}
