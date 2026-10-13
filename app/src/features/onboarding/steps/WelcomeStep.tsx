import wordmark from '@/assets/brand/wordmark.png';
import { Button } from '@/components/Button/Button';

import { onboardingCopy } from '../copy';
import { Pose } from '../Pose';
import { STEP_POSES } from '../steps';
import { StepFrame } from '../StepFrame';

import type { StepViewProps } from './index';

import styles from '../Step.module.css';

const copy = onboardingCopy.welcome;

export function WelcomeStep({ onNext }: StepViewProps) {
  const [first, second] = copy.tagline;

  return (
    <StepFrame
      centered
      title={
        <span className={styles.lockup}>
          <Pose name={STEP_POSES.welcome} />
          <img className={styles.wordmark} src={wordmark} alt={copy.title} />
        </span>
      }
    >
      <p className={styles.tagline}>
        <span>{first}</span>
        <span className={styles.taglineAccent}>{second}</span>
      </p>
      <p className={styles.descriptor}>{copy.descriptor}</p>
      <Button variant="primary" className={styles.start} onClick={onNext}>
        {onboardingCopy.getStarted}
      </Button>
    </StepFrame>
  );
}
