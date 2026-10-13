const REDUCED_MOTION_QUERY = '(prefers-reduced-motion: reduce)';

export const EASE_SPRING = [0.2, 0.8, 0.2, 1] as const;
export const SPRING_POP = { type: 'spring', bounce: 0.45, duration: 0.55 } as const;
export const FADE_RISE = { duration: 0.3, ease: EASE_SPRING } as const;

export function prefersReducedMotion(): boolean {
  return window.matchMedia(REDUCED_MOTION_QUERY).matches;
}
