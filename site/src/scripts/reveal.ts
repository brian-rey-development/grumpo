import { inView } from 'motion';

const REVEAL_DELAY_MS = 500;
const VISIBLE_SHARE = 0.8;

function revealOnce(element: HTMLElement): void {
  const stop = inView(
    element,
    () => {
      window.setTimeout(() => element.toggleAttribute('data-revealed', true), REVEAL_DELAY_MS);
      stop();
    },
    { amount: VISIBLE_SHARE },
  );
}

export function mountReveals(): void {
  document.querySelectorAll<HTMLElement>('[data-reveal]').forEach(revealOnce);
}
