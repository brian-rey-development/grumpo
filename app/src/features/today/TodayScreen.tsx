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
import { formatDayLabel } from '@/lib/format';

import { todayCopy } from './copy';
import { TodayContent } from './TodayContent';
import { useTodayModel } from './useTodayModel';

const DAY_STEP = 1;
const SKELETON_BLOCKS: readonly SkeletonBlock[] = [
  { title: todayCopy.day.title, height: 'var(--size-chart-height)' },
];

export function TodayScreen() {
  const { navigate } = useNavigation();
  const nav = useDateNavigation(DAY_STEP, useRetentionDays());
  const model = useTodayModel(nav.isoDate, nav.isCurrent);
  const dayLabel = formatDayLabel(nav.date, nav.today);
  useToolbarTitle(nav.isCurrent ? null : dayLabel);
  const openSettings = useCallback(() => {
    navigate('settings');
  }, [navigate]);

  return (
    <>
      <DateNavigationActions nav={nav} copy={todayCopy.stepper} />
      <p className="sr-only" role="status">
        {dayLabel}
      </p>
      {model.status === 'loading' ? (
        <UsageSkeleton
          loadingLabel={todayCopy.loading}
          blocks={SKELETON_BLOCKS}
          banner={nav.isCurrent}
          listTitles={[todayCopy.lists.appsTitle, todayCopy.lists.sitesTitle]}
        />
      ) : null}
      {model.status === 'error' ? (
        <LoadFailure
          title={todayCopy.error.title}
          description={usageCopy.loadFailedDescription}
          retryLabel={commonCopy.retry}
          onRetry={model.retry}
        />
      ) : null}
      {model.status === 'ready' ? (
        <TodayContent
          view={model.view}
          date={nav.date}
          isToday={nav.isCurrent}
          stale={model.stale}
          onOpenSettings={openSettings}
        />
      ) : null}
    </>
  );
}
