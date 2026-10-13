import { DateStepper } from '@/components/DateStepper/DateStepper';
import type { DateNavigation } from '@/features/usage/useDateNavigation';
import { PageActions } from '@/layout/PageActions';

interface StepperCopy {
  group: string;
  previous: string;
  next: string;
  reset: string;
}

interface DateNavigationActionsProps {
  nav: DateNavigation;
  copy: StepperCopy;
}

/** The page header controls that step a usage screen back and forward through time. */
export function DateNavigationActions({ nav, copy }: DateNavigationActionsProps) {
  return (
    <PageActions>
      <DateStepper
        label={copy.group}
        previousLabel={copy.previous}
        nextLabel={copy.next}
        canGoNext={!nav.isCurrent}
        canGoPrevious={nav.canGoPrevious}
        onPrevious={nav.previous}
        onNext={nav.next}
        reset={nav.isCurrent ? null : { label: copy.reset, onSelect: nav.reset }}
      />
    </PageActions>
  );
}
