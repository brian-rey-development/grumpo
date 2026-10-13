import { useCallback } from 'react';

import { useLocalHour } from '@/features/usage/useCalendar';
import { type ReportModel, useReportModel } from '@/features/usage/useReportModel';
import type { DayReport } from '@/ipc/generated/DayReport';
import { useDayReport } from '@/ipc/queries';

import { deriveTodayView, type TodayView } from './todayModel';

export function useTodayModel(isoDate: string, isToday: boolean): ReportModel<TodayView> {
  const query = useDayReport(isoDate);
  const hour = useLocalHour();
  const derive = useCallback(
    (report: DayReport) => deriveTodayView(report, isToday, hour),
    [isToday, hour],
  );
  const isOtherDay = useCallback((report: DayReport) => report.date !== isoDate, [isoDate]);
  return useReportModel(query, derive, isOtherDay);
}
