import { type Transition, useReducedMotion } from 'motion/react';

export const spring = {
  snappy: { type: 'spring', stiffness: 520, damping: 40, mass: 0.8 },
  gentle: { type: 'spring', stiffness: 260, damping: 32 },
  squash: { type: 'spring', stiffness: 420, damping: 13, mass: 0.9 },
  pop: { type: 'spring', stiffness: 560, damping: 24, mass: 0.7 },
} as const satisfies Record<string, Transition>;

export const duration = {
  fast: 0.14,
  base: 0.22,
} as const;

const EASE_OUT = [0.2, 0.8, 0.2, 1] as const;
const HUFF_SECONDS = 0.32;
const GROW_SECONDS = 0.2;
const BACKDROP_SECONDS = 0.12;
const TAKEOVER_TEXT_DELAY_SECONDS = 0.4;
const TAKEOVER_STAGGER_SECONDS = 0.07;
const CHART_STAGGER_SECONDS = 0.02;
const LIST_STAGGER_SECONDS = 0.04;

export const tween = {
  fast: { duration: duration.fast, ease: EASE_OUT },
  base: { duration: duration.base, ease: EASE_OUT },
  huff: { duration: HUFF_SECONDS, ease: EASE_OUT },
  grow: { duration: GROW_SECONDS, ease: EASE_OUT },
  backdrop: { duration: BACKDROP_SECONDS, ease: EASE_OUT },
} as const satisfies Record<string, Transition>;

/** The pause between neighbours in an entrance, kept short enough that the whole group lands in about 400ms. */
export function chartStagger(index: number, count: number): number {
  return index * Math.min(CHART_STAGGER_SECONDS, GROW_SECONDS / count);
}

/** When the n-th line of the block screen rises, after the mascot has landed. */
export function takeoverDelay(index: number): number {
  return TAKEOVER_TEXT_DELAY_SECONDS + index * TAKEOVER_STAGGER_SECONDS;
}

export function listStagger(index: number): number {
  return index * LIST_STAGGER_SECONDS;
}

const INSTANT: Transition = { duration: 0 };

export function useMotionTransition(transition: Transition): Transition {
  const reduced = useReducedMotion();
  return reduced ? INSTANT : transition;
}
