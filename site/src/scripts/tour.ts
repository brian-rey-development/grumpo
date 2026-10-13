import { animate } from 'motion';

import { queryAll } from '@/scripts/lib/dom';
import { FADE_RISE, prefersReducedMotion } from '@/scripts/lib/motion';

const ARROW_STEPS: Record<string, number> = {
  ArrowRight: 1,
  ArrowDown: 1,
  ArrowLeft: -1,
  ArrowUp: -1,
};

function select(root: HTMLElement, id: string): void {
  queryAll(root, '[data-tab]').forEach((tab) => {
    const selected = tab.dataset.tab === id;
    tab.setAttribute('aria-selected', String(selected));
    tab.tabIndex = selected ? 0 : -1;
  });
  queryAll(root, '[data-panel]').forEach((panel) => {
    panel.hidden = panel.dataset.panel !== id;
    if (!panel.hidden && !prefersReducedMotion()) {
      animate(panel, { opacity: [0, 1], y: [12, 0] }, FADE_RISE);
    }
  });
}

function moveFocus(
  tabs: HTMLElement[],
  current: HTMLElement,
  step: number,
): HTMLElement | undefined {
  const index = tabs.indexOf(current);
  return tabs[(index + step + tabs.length) % tabs.length];
}

export function mountTour(root: HTMLElement): void {
  const tabs = queryAll(root, '[data-tab]');
  tabs.forEach((tab) => {
    tab.addEventListener('click', () => select(root, tab.dataset.tab ?? ''));
    tab.addEventListener('keydown', (event) => {
      const step = ARROW_STEPS[event.key];
      if (step === undefined) return;
      event.preventDefault();
      const next = moveFocus(tabs, tab, step);
      next?.focus();
      next?.click();
    });
  });
}
