import { onboardingCopy } from '../copy';
import { Pose } from '../Pose';
import { STEP_POSES } from '../steps';
import { StepFrame } from '../StepFrame';

import { RecapList } from './RecapList';

const copy = onboardingCopy.done;

export function DoneStep() {
  return (
    <StepFrame compact hero={<Pose name={STEP_POSES.done} />} title={copy.title} lede={copy.lede}>
      <RecapList />
    </StepFrame>
  );
}
