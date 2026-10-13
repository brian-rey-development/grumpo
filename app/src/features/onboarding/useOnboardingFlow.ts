import { useCallback, useState } from 'react';

import { STEPS, type StepId } from './steps';

const FORWARD = 1;
const BACKWARD = -1;

interface Position {
  index: number;
  direction: typeof FORWARD | typeof BACKWARD;
}

export interface OnboardingFlow {
  step: StepId;
  index: number;
  direction: Position['direction'];
  first: boolean;
  last: boolean;
  next: () => void;
  back: () => void;
}

export function useOnboardingFlow(): OnboardingFlow {
  const [position, setPosition] = useState<Position>({ index: 0, direction: FORWARD });
  const next = useCallback(() => {
    setPosition(({ index }) => ({
      index: Math.min(index + 1, STEPS.length - 1),
      direction: FORWARD,
    }));
  }, []);
  const back = useCallback(() => {
    setPosition(({ index }) => ({ index: Math.max(index - 1, 0), direction: BACKWARD }));
  }, []);

  return {
    step: STEPS[position.index] ?? 'welcome',
    index: position.index,
    direction: position.direction,
    first: position.index === 0,
    last: position.index === STEPS.length - 1,
    next,
    back,
  };
}
