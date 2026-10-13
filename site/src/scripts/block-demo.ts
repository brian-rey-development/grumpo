import { animate } from 'motion';

import { blockDemo } from '@/config/demo';
import { query, queryAll } from '@/scripts/lib/dom';
import { prefersReducedMotion, SPRING_POP } from '@/scripts/lib/motion';
import { formatClock } from '@/scripts/lib/time';

type DemoState = 'idle' | 'blocked' | 'allowed';

const TICK_MS = 1_000;

function isBlocked(host: string): boolean {
  return blockDemo.blocked.some((domain) => host === domain || host.endsWith(`.${domain}`));
}

class BlockDemo {
  private readonly buttons: HTMLButtonElement[];
  private ticker: number | undefined;
  private shows = 0;

  constructor(private readonly root: HTMLElement) {
    this.buttons = queryAll(root, '[data-site]', HTMLButtonElement);
  }

  mount(): void {
    this.buttons.forEach((button) => {
      button.addEventListener('click', () => this.visit(button));
    });
  }

  private text(selector: string, value: string): void {
    query(this.root, selector).textContent = value;
  }

  private visit(button: HTMLButtonElement): void {
    const host = button.dataset.site ?? '';
    this.buttons.forEach((other) => other.setAttribute('aria-pressed', String(other === button)));
    window.clearInterval(this.ticker);
    if (isBlocked(host)) this.block(host);
    else this.allow(host);
  }

  private nextHeadline(): string {
    const { headlines } = blockDemo;
    const headline = headlines[this.shows % headlines.length] ?? headlines[0];
    this.shows += 1;
    return headline;
  }

  private block(host: string): void {
    this.text('[data-headline]', this.nextHeadline());
    this.tick((elapsed) => {
      const left = formatClock(blockDemo.focusSecondsLeft - elapsed);
      this.text('[data-detail]', `${host} is blocked for ${left}.`);
    });
    this.show('blocked');
  }

  private allow(host: string): void {
    this.text('[data-headline]', blockDemo.allowedHeadline);
    this.tick((elapsed) => {
      this.text('[data-detail]', `${host} is allowed. ${formatClock(elapsed)} so far.`);
    });
    this.show('allowed');
  }

  private tick(render: (elapsedSeconds: number) => void): void {
    let elapsed = 0;
    render(elapsed);
    this.ticker = window.setInterval(() => {
      elapsed += 1;
      render(elapsed);
    }, TICK_MS);
  }

  private show(state: DemoState): void {
    this.root.dataset.state = state;
    if (prefersReducedMotion()) return;
    animate(
      query(this.root, `[data-pose="${state}"]`),
      { y: [36, 0], scale: [0.88, 1] },
      SPRING_POP,
    );
    animate(query(this.root, '[data-bubble]'), { scale: [0.8, 1], rotate: [-4, 0] }, SPRING_POP);
  }
}

export function mountBlockDemo(root: HTMLElement): void {
  new BlockDemo(root).mount();
}
