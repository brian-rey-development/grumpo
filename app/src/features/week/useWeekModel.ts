import { useCallback } from 'react';

import { type ReportModel, useReportModel } from '@/features/usage/useReportModel';
import type { WeekReport } from '@/ipc/generated/WeekReport';
import { useWeekReport } from '@/ipc/queries';

import { deriveWeekView, type WeekView } from './weekModel';

export function useWeekModel(endDate: string): ReportModel<WeekView> {
  const query = useWeekReport(endDate);
  const isOtherWeek = useCallback(
    (report: WeekReport) => {
      const lastDay = report.days.at(-1)?.date;
      return lastDay !== undefined && lastDay !== endDate;
    },
    [endDate],
  );
  return useReportModel(query, deriveWeekView, isOtherWeek);
}
