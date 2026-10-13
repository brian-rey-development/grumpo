import { createContext, type ReactNode, useContext, useEffect, useState } from 'react';

import { isDocumentHidden } from '@/lib/documentHidden';

const RevealContext = createContext<Set<string> | null>(null);

export type RevealPhase = 'idle' | 'waiting' | 'play';

type Outcome = 'play' | 'skip';

/** Longer than the slowest entrance, so a mount that is replaced within a blink does not use up the play. */
export const REVEAL_SETTLE_MS = 1000;

/** Remembers which datasets have already played their entrance for the life of the app. */
export function RevealProvider({ children }: { children: ReactNode }) {
  const [revealed] = useState(() => new Set<string>());
  return <RevealContext value={revealed}>{children}</RevealContext>;
}

function useRevealed(): Set<string> {
  const revealed = useContext(RevealContext);
  if (revealed === null) throw new Error('useRevealOnce must be used inside a RevealProvider');
  return revealed;
}

/**
 * Decides once per mount whether a view should play its entrance animation. `idle` means the
 * dataset was already revealed earlier or the window was hidden when the data arrived, `waiting`
 * means it has not been revealed yet but the data is not ready, and `play` means the entrance
 * should run now.
 *
 * A dataset counts as revealed only once its entrance has had time to finish, so a view that
 * remounts straight away still gets to play it.
 *
 * The animation is only an enhancement. A window nobody can see would never run it, so in that case
 * the data is shown as it is and the entrance is skipped for good.
 */
export function useRevealOnce(key: string, ready: boolean): RevealPhase {
  const revealed = useRevealed();
  const [eligible] = useState(() => !revealed.has(key));
  const [outcome, setOutcome] = useState<Outcome | null>(null);
  if (eligible && ready && outcome === null) setOutcome(isDocumentHidden() ? 'skip' : 'play');

  useEffect(() => {
    if (outcome === null) return undefined;
    if (outcome === 'skip') {
      revealed.add(key);
      return undefined;
    }
    const timer = window.setTimeout(() => {
      revealed.add(key);
    }, REVEAL_SETTLE_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, [revealed, key, outcome]);

  if (!eligible || outcome === 'skip') return 'idle';
  return outcome === 'play' ? 'play' : 'waiting';
}
