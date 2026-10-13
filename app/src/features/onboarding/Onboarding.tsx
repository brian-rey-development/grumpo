import { useIsMutating } from '@tanstack/react-query';
import { AnimatePresence, motion, useReducedMotion } from 'motion/react';
import { useCallback, useEffect, useRef } from 'react';

import { Button } from '@/components/Button/Button';
import { commonCopy } from '@/copy/common';
import { describeFailure } from '@/copy/failures';
import { spring, tween, useMotionTransition } from '@/design/motion';
import { useCompleteOnboarding } from '@/ipc/mutations';
import { useNavigation } from '@/layout/navigation';

import { onboardingCopy } from './copy';
import { preloadPose } from './Pose';
import { ProgressBar } from './ProgressBar';
import { STEP_VIEWS } from './steps/index';
import { type Destination, STEP_POSES, type StepId, STEPS } from './steps';
import { type OnboardingFlow, useOnboardingFlow } from './useOnboardingFlow';
import { useContentBelow } from './useContentBelow';
import { useEnterToAdvance } from './useEnterToAdvance';
import { useStepSettled } from './useStepSettled';

import styles from './Onboarding.module.css';

const SLIDE_OFFSET_PX = 28;
const CENTERED: readonly StepId[] = ['welcome'];

function nextLabel(settled: boolean): string {
  return settled ? onboardingCopy.next : onboardingCopy.later;
}

interface StepStageProps {
  flow: OnboardingFlow;
}

function StepStage({ flow }: StepStageProps) {
  const reduced = useReducedMotion() === true;
  const offset = reduced ? 0 : SLIDE_OFFSET_PX;
  const StepView = STEP_VIEWS[flow.step];

  return (
    <div className={styles.stage}>
      <AnimatePresence initial={false} custom={flow.direction}>
        <motion.div
          key={flow.step}
          className={styles.column}
          data-align={CENTERED.includes(flow.step) ? 'center' : 'start'}
          custom={flow.direction}
          variants={{
            enter: (direction: number) => ({ opacity: 0, x: direction * offset }),
            center: { opacity: 1, x: 0, transition: reduced ? tween.base : spring.snappy },
            exit: (direction: number) => ({
              opacity: 0,
              x: -direction * offset,
              transition: tween.fast,
            }),
          }}
          initial="enter"
          animate="center"
          exit="exit"
        >
          <StepView onNext={flow.next} />
        </motion.div>
      </AnimatePresence>
    </div>
  );
}

interface FooterProps {
  flow: OnboardingFlow;
  settled: boolean;
  finishing: boolean;
  onFinish: (destination: Destination) => void;
}

function FooterActions({ flow, settled, finishing, onFinish }: FooterProps) {
  if (flow.last) {
    return (
      <>
        <Button
          disabled={finishing}
          onClick={() => {
            onFinish('today');
          }}
        >
          {onboardingCopy.goToday}
        </Button>
        <Button
          variant="primary"
          loading={finishing}
          onClick={() => {
            onFinish('rules');
          }}
        >
          {onboardingCopy.addRule}
        </Button>
      </>
    );
  }
  if (flow.first) return null;
  return (
    <Button variant={settled ? 'primary' : 'secondary'} className={styles.next} onClick={flow.next}>
      {nextLabel(settled)}
    </Button>
  );
}

function BackButton({ flow }: { flow: OnboardingFlow }) {
  const fade = useMotionTransition(tween.fast);

  return (
    <motion.div
      className={styles.back}
      initial={false}
      animate={{ opacity: flow.first ? 0 : 1 }}
      transition={fade}
      inert={flow.first}
    >
      <Button variant="plain" onClick={flow.back}>
        {onboardingCopy.back}
      </Button>
    </motion.div>
  );
}

function usePreloadNextPose(index: number): void {
  useEffect(() => {
    const upcoming = STEPS[index + 1];
    if (upcoming !== undefined) preloadPose(STEP_POSES[upcoming]);
  }, [index]);
}

export function Onboarding() {
  const flow = useOnboardingFlow();
  usePreloadNextPose(flow.index);
  const settled = useStepSettled(flow.step);
  const complete = useCompleteOnboarding();
  const { navigate } = useNavigation();
  const { mutateAsync } = complete;

  const finish = useCallback(
    (destination: Destination) => {
      void mutateAsync().then(
        () => {
          navigate(destination);
        },
        () => undefined,
      );
    },
    [mutateAsync, navigate],
  );
  const busy = useIsMutating() > 0;
  const advance = useCallback(() => {
    if (complete.isPending) return;
    if (flow.last) finish('rules');
    else flow.next();
  }, [flow, finish, complete.isPending]);
  useEnterToAdvance(advance, busy);
  const scroller = useRef<HTMLElement>(null);
  const contentBelow = useContentBelow(scroller);

  return (
    <div className={styles.root}>
      <div className={styles.drag} data-tauri-drag-region />
      <main ref={scroller} className={styles.main}>
        <StepStage flow={flow} />
      </main>
      <footer className={styles.footer} data-edge={contentBelow}>
        <BackButton flow={flow} />
        <ProgressBar index={flow.index} />
        <div className={styles.actions}>
          {complete.error === null ? null : (
            <p className={styles.error} role="alert">
              {`${onboardingCopy.saveFailed} ${describeFailure(complete.error, commonCopy.tryAgainSoon)}`}
            </p>
          )}
          <FooterActions
            flow={flow}
            settled={settled}
            finishing={complete.isPending}
            onFinish={finish}
          />
        </div>
      </footer>
    </div>
  );
}
