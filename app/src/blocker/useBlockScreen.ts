import { useCallback, useRef, useState } from 'react';

import { invokeCommand } from '@/ipc/commands';
import type { BlockScreen } from '@/ipc/generated/BlockScreen';
import { useBackendEvent } from '@/ipc/useBackendEvent';

/**
 * What the window is showing, or null while it is hidden. The command read covers a block that
 * happened before the listener was live, and any newer event, including the hide, wins over a
 * read that was still in flight.
 */
export function useBlockScreen(): BlockScreen | null {
  const [screen, setScreen] = useState<BlockScreen | null>(null);
  const events = useRef(0);

  const load = useCallback(() => {
    const seen = events.current;
    invokeCommand('get_block_screen')
      .then((loaded) => {
        if (loaded !== null && events.current === seen) setScreen(loaded);
      })
      .catch(() => undefined);
  }, []);

  useBackendEvent(
    'block-screen',
    (shown) => {
      events.current += 1;
      setScreen(shown);
    },
    load,
  );

  useBackendEvent('block-screen-hidden', () => {
    events.current += 1;
    setScreen(null);
  });

  return screen;
}

export function dismissBlockScreen(id: number): void {
  invokeCommand('dismiss_block_screen', { id }).catch(() => undefined);
}
