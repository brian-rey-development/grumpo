import { ChevronLeft, ChevronRight } from 'lucide-react';
import { useCallback, useRef } from 'react';

import { Button } from '@/components/Button/Button';
import { IconButton } from '@/components/IconButton/IconButton';
import { type StepDirection, useStepShortcuts } from './useStepShortcuts';

import styles from './DateStepper.module.css';

const ICON_SIZE = 18;
const PREVIOUS_SHORTCUT = 'Meta+[';
const NEXT_SHORTCUT = 'Meta+]';

interface DateStepperReset {
  label: string;
  onSelect: () => void;
}

interface DateStepperProps {
  label: string;
  previousLabel: string;
  nextLabel: string;
  canGoNext: boolean;
  /** False when there is no older history to show, which is the case beyond the retention period. */
  canGoPrevious?: boolean;
  onPrevious: () => void;
  onNext: () => void;
  /** Offered only while away from the present, so it doubles as the "you are in the past" cue. */
  reset: DateStepperReset | null;
}

export function DateStepper({
  label,
  previousLabel,
  nextLabel,
  canGoNext,
  canGoPrevious = true,
  onPrevious,
  onNext,
  reset,
}: DateStepperProps) {
  const onStep = useCallback(
    (direction: StepDirection) => {
      if (direction === 'previous') {
        if (canGoPrevious) onPrevious();
      } else if (canGoNext) onNext();
    },
    [canGoNext, canGoPrevious, onPrevious, onNext],
  );
  useStepShortcuts(onStep);
  const previousRef = useRef<HTMLButtonElement>(null);

  function selectReset(): void {
    previousRef.current?.focus();
    reset?.onSelect();
  }

  return (
    <div className={styles.stepper} role="group" aria-label={label}>
      {reset === null ? null : (
        <Button size="small" onClick={selectReset}>
          {reset.label}
        </Button>
      )}
      <div className={styles.pair}>
        <IconButton
          className={styles.step}
          ref={previousRef}
          label={previousLabel}
          aria-keyshortcuts={PREVIOUS_SHORTCUT}
          aria-disabled={!canGoPrevious || undefined}
          icon={<ChevronLeft size={ICON_SIZE} aria-hidden="true" />}
          onClick={canGoPrevious ? onPrevious : undefined}
        />
        <IconButton
          className={styles.step}
          label={nextLabel}
          aria-keyshortcuts={NEXT_SHORTCUT}
          aria-disabled={!canGoNext || undefined}
          icon={<ChevronRight size={ICON_SIZE} aria-hidden="true" />}
          onClick={canGoNext ? onNext : undefined}
        />
      </div>
    </div>
  );
}
