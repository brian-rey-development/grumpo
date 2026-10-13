import { useMemo } from 'react';

export type ReportModel<View> =
  | { status: 'loading' }
  | { status: 'error'; retry: () => void }
  | { status: 'ready'; view: View; stale: boolean };

interface ReportQuery<Data> {
  data: Data | undefined;
  error: Error | null;
  isPlaceholderData: boolean;
  refetch: () => Promise<unknown>;
}

/**
 * Shapes a report query for a screen. `stale` is true while the previous period is still on show
 * because the requested one has not arrived yet. Pass `derive` and `isOtherPeriod` memoized.
 */
export function useReportModel<Data, View>(
  { data, error, isPlaceholderData, refetch }: ReportQuery<Data>,
  derive: (data: Data) => View,
  isOtherPeriod: (data: Data) => boolean,
): ReportModel<View> {
  return useMemo<ReportModel<View>>(() => {
    if (data !== undefined) {
      return {
        status: 'ready',
        view: derive(data),
        stale: isPlaceholderData || isOtherPeriod(data),
      };
    }
    if (error !== null) {
      return {
        status: 'error',
        retry: () => {
          void refetch();
        },
      };
    }
    return { status: 'loading' };
  }, [data, error, isPlaceholderData, refetch, derive, isOtherPeriod]);
}
