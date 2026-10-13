import { queryAll } from '@/scripts/lib/dom';
import { UNLOCK_STEP_EVENT } from '@/scripts/lib/events';

type RungStatus = 'done' | 'current' | 'later';

function statusOf(index: number, current: number): RungStatus {
  if (index < current) return 'done';
  return index === current ? 'current' : 'later';
}

function render(rungs: HTMLElement[], current: number): void {
  rungs.forEach((rung) => {
    const status = statusOf(Number(rung.dataset.ladderStep), current);
    rung.dataset.status = status;
    if (status === 'current') rung.setAttribute('aria-current', 'step');
    else rung.removeAttribute('aria-current');
  });
}

export function mountLadder(root: HTMLElement): void {
  const rungs = queryAll(root, '[data-ladder-step]');
  root.addEventListener(UNLOCK_STEP_EVENT, (event) => {
    if (event instanceof CustomEvent && typeof event.detail === 'number')
      render(rungs, event.detail);
  });
}
