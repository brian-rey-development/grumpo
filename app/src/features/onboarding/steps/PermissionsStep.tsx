import { PermissionsSection } from '@/features/settings/PermissionsSection';

import { onboardingCopy } from '../copy';
import { Pose } from '../Pose';
import { STEP_POSES } from '../steps';
import { StepFrame } from '../StepFrame';

const copy = onboardingCopy.permissions;

export function PermissionsStep() {
  return (
    <StepFrame
      compact
      hero={<Pose name={STEP_POSES.permissions} />}
      title={copy.title}
      lede={copy.lede}
    >
      <PermissionsSection variant="bare" />
    </StepFrame>
  );
}
