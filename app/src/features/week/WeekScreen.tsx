import { useCallback } from 'react';

import { LoadFailure } from '@/components/LoadFailure/LoadFailure';
import { commonCopy } from '@/copy/common';
import { usageCopy } from '@/features/usage/copy';
import { DateNavigationActions } from '@/features/usage/DateNavigationActions';
import { useDateNavigation } from '@/features/usage/useDateNavigation';
import { useRetentionDays } from '@/features/usage/useRetentionDays';
import { type SkeletonBlock, UsageSkeleton } from '@/features/usage/UsageSkeleton';
import { useNavigation } from '@/layout/navigation';
import { useToolbarTitle } from '@/layout/toolbarTitle';
import { formatMonthDay, formatTimeRange } from '@/lib/format';
import { addDays, DAYS_PER_WEEK } from '@/lib/time';

import { weekCopy } from './copy';
import { useWeekModel } from './useWeekModel';
import { WeekContent } from './WeekContent';

const SKELETON_BLOCKS: readonly SkeletonBlock[] = [
  { title: weekCopy.sections.daily, height: 'var(--size-chart-height)' },
];

export function WeekScreen() {
  const { navigate } = useNavigation();
  const nav = useDateNavigation(DAYS_PER_WEEK, useRetentionDays());
  const model = useWeekModel(nav.isoDate);
  const title = nav.isCurrent
    ? weekCopy.currentTitle
    : formatTimeRange(
        formatMonthDay(addDays(nav.date, 1 - DAYS_PER_WEEK)),
        formatMonthDay(nav.date),
      );
  useToolbarTitle(title);
  const openSettings = useCallback(() => {
    navigate('settings');
  }, [navigate]);

  return (
    <>
      <DateNavigationActions nav={nav} copy={weekCopy.stepper} />
      <p className="sr-only" role="status">
        {title}
      </p>
      {model.status === 'loading' ? (
        <UsageSkeleton
          loadingLabel={weekCopy.loading}
          blocks={SKELETON_BLOCKS}
          listTitles={[weekCopy.lists.appsTitle, weekCopy.lists.sitesTitle]}
        />
      ) : null}
      {model.status === 'error' ? (
        <LoadFailure
          title={weekCopy.error.title}
          description={usageCopy.loadFailedDescription}
          retryLabel={commonCopy.retry}
          onRetry={model.retry}
        />
      ) : null}
      {model.status === 'ready' ? (
        <WeekContent
          view={model.view}
          today={nav.today}
          isCurrent={nav.isCurrent}
          stale={model.stale}
          onOpenSettings={openSettings}
        />
      ) : null}
    </>
  );
}
