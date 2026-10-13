import type { ComponentType } from 'react';

import type { StepId } from '../steps';

import { DoneStep } from './DoneStep';
import { HowItWorksStep } from './HowItWorksStep';
import { PermissionsStep } from './PermissionsStep';
import { PrivacyStep } from './PrivacyStep';
import { ServiceStep } from './ServiceStep';
import { WelcomeStep } from './WelcomeStep';

export interface StepViewProps {
  onNext: () => void;
}

export const STEP_VIEWS: Record<StepId, ComponentType<StepViewProps>> = {
  welcome: WelcomeStep,
  how: HowItWorksStep,
  private: PrivacyStep,
  service: ServiceStep,
  permissions: PermissionsStep,
  done: DoneStep,
};
