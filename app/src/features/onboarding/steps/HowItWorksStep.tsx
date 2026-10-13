import { onboardingCopy } from '../copy';
import { ObjectPoint } from '../ObjectPoint';
import { Pose } from '../Pose';
import { STEP_POSES } from '../steps';
import { StepFrame } from '../StepFrame';

import styles from '../Step.module.css';

const copy = onboardingCopy.how;

export function HowItWorksStep() {
  return (
    <StepFrame hero={<Pose name={STEP_POSES.how} />} title={copy.title}>
      <ul className={styles.points}>
        {copy.points.map((point) => (
          <ObjectPoint
            key={point.title}
            object={point.object}
            title={point.title}
            body={point.body}
          />
        ))}
      </ul>
      <p className={styles.rule}>
        <span>{copy.ruleLead}</span> {copy.rule}
      </p>
    </StepFrame>
  );
}
