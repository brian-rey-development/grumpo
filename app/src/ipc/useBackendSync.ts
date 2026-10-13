import { useQueryClient } from '@tanstack/react-query';
import { useEffect } from 'react';

import { invalidateUsage } from '@/ipc/mutations';
import { queryKeys } from '@/ipc/queryKeys';
import { useBackendEvent } from '@/ipc/useBackendEvent';
import { isDocumentHidden } from '@/lib/documentHidden';

/**
 * Keeps the cache in step with the backend. While the window is hidden, changes only mark queries
 * stale, because refetching every sample for a window nobody can see wastes work. Showing the
 * window again refetches whatever went stale.
 */
export function useBackendSync(): void {
  const queryClient = useQueryClient();

  useEffect(() => {
    function onVisibilityChange(): void {
      if (isDocumentHidden()) return;
      void queryClient.refetchQueries({ type: 'active', stale: true });
    }
    document.addEventListener('visibilitychange', onVisibilityChange);
    return () => {
      document.removeEventListener('visibilitychange', onVisibilityChange);
    };
  }, [queryClient]);

  const refreshUsage = () => {
    void invalidateUsage(queryClient, isDocumentHidden() ? 'none' : 'active');
  };

  useBackendEvent('usage-changed', refreshUsage, refreshUsage);

  useBackendEvent(
    'status-changed',
    (status) => {
      queryClient.setQueryData(queryKeys.focusStatus, status);
    },
    () => {
      void queryClient.invalidateQueries({
        queryKey: queryKeys.focusStatus,
        refetchType: isDocumentHidden() ? 'none' : 'active',
      });
    },
  );
}
