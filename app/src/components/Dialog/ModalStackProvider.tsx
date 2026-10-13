import { type ReactNode, useCallback, useMemo, useRef, useState } from 'react';

import { ModalStackContext } from '@/components/Dialog/modalStack';

import styles from './ModalStackProvider.module.css';

/**
 * Tracks open dialogs and keeps the app behind them inert. Dialogs render in a portal outside
 * this wrapper, so they stay interactive while everything else is blocked.
 */
export function ModalStackProvider({ children }: { children: ReactNode }) {
  const rootRef = useRef<HTMLDivElement>(null);
  const stackRef = useRef<readonly string[]>([]);
  const [stack, setStack] = useState<readonly string[]>([]);

  const commit = useCallback((next: readonly string[]) => {
    stackRef.current = next;
    rootRef.current?.toggleAttribute('inert', next.length > 0);
    setStack(next);
  }, []);

  const register = useCallback(
    (id: string) => {
      commit([...stackRef.current, id]);
      return () => {
        commit(stackRef.current.filter((entry) => entry !== id));
      };
    },
    [commit],
  );

  const value = useMemo(
    () => ({ topId: stack.at(-1) ?? null, depth: stack.length, register }),
    [stack, register],
  );

  return (
    <ModalStackContext value={value}>
      <div ref={rootRef} className={styles.root}>
        {children}
      </div>
    </ModalStackContext>
  );
}
