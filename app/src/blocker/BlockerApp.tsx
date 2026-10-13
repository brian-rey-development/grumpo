import { MotionConfig } from 'motion/react';
import { useEffect } from 'react';

import { preloadMascots } from '@/components/Mascot/Mascot';

import { BlockScreenView } from './BlockScreenView';
import { dismissBlockScreen, useBlockScreen } from './useBlockScreen';

import styles from './BlockerApp.module.css';

preloadMascots(['armsCrossedLarge', 'pushLarge']);

function useEscapeToDismiss(id: number | undefined): void {
  useEffect(() => {
    if (id === undefined) return;
    function onKeyDown(event: KeyboardEvent): void {
      if (event.key === 'Escape' && id !== undefined) dismissBlockScreen(id);
    }
    window.addEventListener('keydown', onKeyDown);
    return () => {
      window.removeEventListener('keydown', onKeyDown);
    };
  }, [id]);
}

export function BlockerApp() {
  const screen = useBlockScreen();
  useEscapeToDismiss(screen?.id);

  return (
    <MotionConfig reducedMotion="user">
      {screen === null ? (
        <div className={styles.idle} />
      ) : (
        <BlockScreenView
          key={screen.id}
          screen={screen}
          onDismiss={() => {
            dismissBlockScreen(screen.id);
          }}
        />
      )}
    </MotionConfig>
  );
}
