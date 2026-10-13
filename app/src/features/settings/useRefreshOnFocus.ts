import { useEffect } from 'react';

/** Permissions are granted in System Settings, so look again whenever the window comes back. */
export function useRefreshOnFocus(refresh: () => void): void {
  useEffect(() => {
    window.addEventListener('focus', refresh);
    return () => {
      window.removeEventListener('focus', refresh);
    };
  }, [refresh]);
}
