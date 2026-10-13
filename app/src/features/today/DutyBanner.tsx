import { AnimatePresence, motion, useReducedMotion } from 'motion/react';

import { Button } from '@/components/Button/Button';
import { Mascot, type MascotPose, preloadMascots } from '@/components/Mascot/Mascot';
import { Skeleton } from '@/components/Skeleton/Skeleton';
import { commonCopy } from '@/copy/common';
import { spring, tween, useMotionTransition } from '@/design/motion';
import { deriveFocus, hasLiveDeadline } from '@/features/status/focusState';
import { useUnlock } from '@/features/unlock/unlockContext';
import { useClockOptions, useFocusStatus } from '@/ipc/queries';
import { useNavigation } from '@/layout/navigation';
import { MS_PER_MINUTE, MS_PER_SECOND } from '@/lib/time';
import { useNow } from '@/lib/useNow';

import {
  type BannerAction,
  type BannerModel,
  type BannerTone,
  describeBanner,
} from './bannerModel';
import { todayCopy } from './copy';

import styles from './DutyBanner.module.css';

const copy = todayCopy.banner;
const TONES: readonly BannerTone[] = ['focus', 'off', 'limit', 'unlocked', 'unavailable'];
const POPPED = { scale: 0.6, opacity: 0 } as const;

preloadMascots(['laptop', 'hourglass', 'sleep', 'armsCrossed', 'zen', 'stop']);

const LABELS: Record<BannerAction, string> = {
  editRules: copy.actions.editRules,
  editSchedule: copy.actions.editSchedule,
  unlock: copy.actions.unlock,
  relock: copy.actions.relock,
  openSettings: commonCopy.openSettings,
};

function useBannerActions(): Record<BannerAction, () => void> {
  const { navigate } = useNavigation();
  const unlock = useUnlock();
  const openUnlock = () => {
    unlock.open();
  };
  return {
    editRules: () => {
      navigate('rules');
    },
    editSchedule: () => {
      navigate('rules');
    },
    openSettings: () => {
      navigate('settings');
    },
    unlock: openUnlock,
    relock: openUnlock,
  };
}

function Sticker({ pose }: { pose: MascotPose }) {
  const reduced = useReducedMotion();
  const pop = useMotionTransition(spring.pop);
  const fade = useMotionTransition(tween.fast);
  const hidden = reduced ? { opacity: 0 } : POPPED;

  return (
    <div className={styles.sticker}>
      <AnimatePresence initial={false}>
        <motion.div
          key={pose}
          className={styles.pose}
          initial={hidden}
          animate={{ scale: 1, opacity: 1, transition: pop }}
          exit={{ ...hidden, transition: fade }}
        >
          <Mascot pose={pose} size="badge" />
        </motion.div>
      </AnimatePresence>
    </div>
  );
}

interface ContentProps {
  model: BannerModel;
  handlers: Record<BannerAction, () => void>;
}

function Content({ model, handlers }: ContentProps) {
  return (
    <>
      <div className={styles.text}>
        <h2 className={styles.headline}>{model.headline}</h2>
        <p className={styles.line}>{model.line}</p>
      </div>
      <div className={styles.actions}>
        {model.actions.map((action, index) => (
          <Button
            key={action}
            variant="inverse"
            className={index === 0 ? undefined : styles.ghost}
            onClick={handlers[action]}
          >
            {LABELS[action]}
          </Button>
        ))}
      </div>
    </>
  );
}

function BannerFrame({ model, handlers }: ContentProps) {
  const fade = useMotionTransition(tween.base);

  return (
    <div className={styles.frame}>
      <section className={styles.banner} aria-label={copy.label}>
        {TONES.map((tone) => (
          <span
            key={tone}
            className={styles.fill}
            data-tone={tone}
            data-active={tone === model.tone || undefined}
            aria-hidden="true"
          />
        ))}
        <Sticker pose={model.pose} />
        <div className={styles.swap}>
          <AnimatePresence initial={false}>
            <motion.div
              key={model.tone}
              className={styles.content}
              data-tone={model.tone}
              initial={{ opacity: 0 }}
              animate={{ opacity: 1, transition: fade }}
              exit={{ opacity: 0, transition: fade }}
            >
              <Content model={model} handlers={handlers} />
            </motion.div>
          </AnimatePresence>
        </div>
      </section>
    </div>
  );
}

export function DutyBanner() {
  const query = useFocusStatus();
  const now = useNow(hasLiveDeadline(query.data) ? MS_PER_SECOND : MS_PER_MINUTE);
  const clock = useClockOptions();
  const handlers = useBannerActions();

  if (query.isPending) {
    return <Skeleton height="var(--size-banner)" radius="var(--radius-xl)" />;
  }

  const { state, unlockWait } = deriveFocus(query.data, now);
  const rules = query.data?.kind === 'connected' ? query.data.status.rules : null;
  const model = describeBanner({ state, unlockWait, rules, now, clock });

  return <BannerFrame model={model} handlers={handlers} />;
}
