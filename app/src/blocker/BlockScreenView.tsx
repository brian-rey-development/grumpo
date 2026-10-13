import { Lock } from 'lucide-react';
import { AnimatePresence, motion, useReducedMotion } from 'motion/react';
import { useEffect, useId, useRef, useState } from 'react';

import { Button } from '@/components/Button/Button';
import { Mascot, type MascotPose } from '@/components/Mascot/Mascot';
import { spring, tween } from '@/design/motion';
import type { BlockScreen } from '@/ipc/generated/BlockScreen';

import { bodyFor, headlineFor, toneFor } from './content';
import { blockerCopy } from './copy';
import { Countdown } from './Countdown';
import { EntranceProvider, Rise } from './Entrance';

import styles from './BlockScreenView.module.css';

const ICON_SIZE = 16;
const SQUASH_FROM = { scaleX: 1.12, scaleY: 0.78, y: 10 } as const;
const AT_REST = { opacity: 1, scaleX: 1, scaleY: 1, y: 0 } as const;
const HUFF_DELAY_SECONDS = 0.36;
const HUFF_SCALE = [1, 1.05, 1];

// Focus lands on the button so Enter works, without a keyboard ring on a screen nobody tabbed into.
const QUIET_FOCUS: FocusOptions & { focusVisible: boolean } = { focusVisible: false };

function useImageDecoded() {
  const ref = useRef<HTMLDivElement>(null);
  const [decoded, setDecoded] = useState(false);

  useEffect(() => {
    const image = ref.current?.querySelector('img');
    const ready = () => {
      setDecoded(true);
    };
    if (!image || !('decode' in image)) {
      ready();
      return;
    }
    image.decode().then(ready, ready);
  }, []);

  return { ref, decoded };
}

function AnimatedMascot({ pose }: { pose: MascotPose }) {
  const reduced = useReducedMotion();
  const { ref, decoded } = useImageDecoded();
  const hidden = reduced ? { opacity: 0 } : { opacity: 0, ...SQUASH_FROM };

  return (
    <motion.div
      ref={ref}
      className={styles.stage}
      initial={hidden}
      animate={decoded ? AT_REST : hidden}
      transition={{ default: spring.squash, opacity: tween.fast }}
    >
      <motion.div
        className={styles.huff}
        animate={{ scale: reduced || !decoded ? 1 : HUFF_SCALE }}
        transition={{ ...tween.huff, delay: HUFF_DELAY_SECONDS }}
      >
        <Mascot pose={pose} size="takeover" />
      </motion.div>
    </motion.div>
  );
}

function StrictNote() {
  return (
    <p className={styles.strict}>
      <Lock className={styles.lock} size={ICON_SIZE} fill="currentColor" aria-hidden="true" />
      {blockerCopy.strict}
    </p>
  );
}

function poseFor({ reason }: BlockScreen): MascotPose {
  return reason === 'limit' ? 'armsCrossedLarge' : 'pushLarge';
}

interface BlockScreenViewProps {
  screen: BlockScreen;
  onDismiss: () => void;
}

export function BlockScreenView({ screen, onDismiss }: BlockScreenViewProps) {
  const dismissRef = useRef<HTMLButtonElement>(null);
  const headlineId = useId();
  const bodyId = useId();

  useEffect(() => {
    dismissRef.current?.focus(QUIET_FOCUS);
  }, []);

  return (
    <motion.main
      className={styles.screen}
      data-tone={toneFor(screen)}
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={tween.backdrop}
    >
      <EntranceProvider>
        <div className={styles.column}>
          <AnimatedMascot pose={poseFor(screen)} />
          <Rise order={0} className={styles.copy}>
            <h1 id={headlineId} className={styles.headline}>
              {headlineFor(screen.id)}
            </h1>
            <p id={bodyId} className={styles.body}>
              {bodyFor(screen)}
            </p>
          </Rise>
          <div className={styles.status}>
            <AnimatePresence>
              {screen.until === null ? null : (
                <Rise key="countdown" order={1} className={styles.panel}>
                  <Countdown until={screen.until} reason={screen.reason} />
                </Rise>
              )}
              {screen.strict ? (
                <Rise key="strict" order={2}>
                  <StrictNote />
                </Rise>
              ) : null}
            </AnimatePresence>
          </div>
          <Rise order={3} className={styles.actions}>
            <Button
              ref={dismissRef}
              variant="primary"
              className={styles.dismiss}
              aria-describedby={`${headlineId} ${bodyId}`}
              onClick={onDismiss}
            >
              {blockerCopy.dismiss}
            </Button>
            <p className={styles.footer}>{blockerCopy.footer}</p>
          </Rise>
        </div>
      </EntranceProvider>
    </motion.main>
  );
}
