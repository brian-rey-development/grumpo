import { QueryClient } from '@tanstack/react-query';

import { MS_PER_MINUTE, MS_PER_SECOND } from '@/lib/time';

const STALE_TIME_MS = 5 * MS_PER_SECOND;
const GC_TIME_MS = 5 * MS_PER_MINUTE;
const QUERY_RETRIES = 1;

export function createQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: {
        staleTime: STALE_TIME_MS,
        gcTime: GC_TIME_MS,
        retry: QUERY_RETRIES,
        refetchOnWindowFocus: false,
        refetchOnReconnect: false,
        networkMode: 'always',
      },
      mutations: { networkMode: 'always', retry: false },
    },
  });
}

export const queryClient = createQueryClient();
