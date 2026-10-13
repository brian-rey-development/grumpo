import { animate } from 'motion';

import { unlockDemo } from '@/config/demo';
import { query, queryAll } from '@/scripts/lib/dom';
import { UNLOCK_STEP_EVENT } from '@/scripts/lib/events';
import { FADE_RISE, prefersReducedMotion } from '@/scripts/lib/motion';
import { formatClock, minutesToSeconds } from '@/scripts/lib/time';

type UnlockState = 'idle' | 'cooling' | 'typing' | 'choosing' | 'unlocked';
type CharStatus = 'ok' | 'bad' | 'next' | 'pending';

const MS_PER_SECOND = 1_000;
const FRAME_MS = 100;
const LAST_STEP = unlockDemo.cooldowns.length - 1;

function cooldownFor(step: number): number {
  const rung = unlockDemo.cooldowns[Math.min(step, LAST_STEP)];
  if (rung === undefined) throw new Error(`No cooldown for step ${step}`);
  return rung.minutes;
}

function charStatus(typed: string, index: number): CharStatus {
  if (index === typed.length) return 'next';
  if (index > typed.length) return 'pending';
  return typed[index] === unlockDemo.sentence[index] ? 'ok' : 'bad';
}

function clockAfter(minutes: number): string {
  const until = new Date(Date.now() + minutesToSeconds(minutes) * MS_PER_SECOND);
  return until.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

class UnlockDemo {
  private readonly typed: HTMLInputElement;
  private readonly chars: HTMLElement[];
  private timer: number | undefined;
  private step = 0;

  constructor(private readonly root: HTMLElement) {
    this.typed = query(root, '[data-typed]', HTMLInputElement);
    this.chars = queryAll(root, '[data-char]');
  }

  mount(): void {
    this.on('[data-request]', () => this.startCooldown());
    this.on('[data-cancel]', () => this.setState('idle', '[data-request]'));
    this.on('[data-give-up]', () => this.setState('idle', '[data-request]'));
    this.on('[data-relock]', () => this.relock());
    queryAll(this.root, '[data-minutes]', HTMLButtonElement).forEach((button) => {
      button.addEventListener('click', () => this.unlock(Number(button.dataset.minutes)));
    });
    this.typed.addEventListener('input', () => this.checkSentence());
    this.typed.addEventListener('paste', (event) => event.preventDefault());
    this.typed.addEventListener('drop', (event) => event.preventDefault());
  }

  private on(selector: string, handler: () => void): void {
    query(this.root, selector, HTMLButtonElement).addEventListener('click', handler);
  }

  private setState(state: UnlockState, focus: string): void {
    window.clearInterval(this.timer);
    this.root.dataset.state = state;
    query(this.root, focus).focus({ preventScroll: true });
    if (prefersReducedMotion()) return;
    animate(query(this.root, `[data-panel="${state}"]`), { opacity: [0, 1], y: [8, 0] }, FADE_RISE);
  }

  private startCooldown(): void {
    const totalMs = (minutesToSeconds(cooldownFor(this.step)) * MS_PER_SECOND) / unlockDemo.speedup;
    const startedAt = performance.now();
    this.setState('cooling', '[data-cancel]');
    this.renderCooldown(0, totalMs);
    this.timer = window.setInterval(() => {
      const elapsed = performance.now() - startedAt;
      if (elapsed >= totalMs) this.openSentence();
      else this.renderCooldown(elapsed, totalMs);
    }, FRAME_MS);
  }

  private renderCooldown(elapsedMs: number, totalMs: number): void {
    const remainingSeconds = ((totalMs - elapsedMs) * unlockDemo.speedup) / MS_PER_SECOND;
    const clock = formatClock(Math.ceil(remainingSeconds));
    const display = query(this.root, '[data-cooldown]');
    if (display.textContent !== clock) display.textContent = clock;
    query(this.root, '[data-progress]').style.transform = `scaleX(${elapsedMs / totalMs})`;
  }

  private openSentence(): void {
    this.typed.value = '';
    this.checkSentence();
    this.setState('typing', '[data-typed]');
  }

  private checkSentence(): void {
    const typed = this.typed.value;
    this.chars.forEach((char, index) => {
      char.dataset.status = charStatus(typed, index);
    });
    if (typed === unlockDemo.sentence) this.setState('choosing', '[data-minutes]');
  }

  private unlock(minutes: number): void {
    const next = cooldownFor(this.step + 1);
    query(this.root, '[data-unlocked]').textContent = `Unlocked for ${minutes} minutes.`;
    query(this.root, '[data-next]').textContent =
      `Back on duty at ${clockAfter(minutes)}. Next unlock waits ${next} minutes.`;
    this.setState('unlocked', '[data-relock]');
  }

  private relock(): void {
    this.step = Math.min(this.step + 1, LAST_STEP);
    this.root.dispatchEvent(
      new CustomEvent(UNLOCK_STEP_EVENT, { bubbles: true, detail: this.step }),
    );
    this.setState('idle', '[data-request]');
  }
}

export function mountUnlockDemo(root: HTMLElement): void {
  new UnlockDemo(root).mount();
}
