import { focusManager } from '@tanstack/react-query';

/**
 * TanStack only listens for visibilitychange, which a webview skips when the user merely switches
 * to another app and back. Listening for the window's own focus lets queries that opt in with
 * refetchOnWindowFocus catch up, such as the clock style after a change in System Settings.
 */
export function watchWindowFocus(): void {
  focusManager.setEventListener((handleFocus) => {
    const onChange = () => {
      handleFocus();
    };
    window.addEventListener('visibilitychange', onChange);
    window.addEventListener('focus', onChange);
    return () => {
      window.removeEventListener('visibilitychange', onChange);
      window.removeEventListener('focus', onChange);
    };
  });
}
