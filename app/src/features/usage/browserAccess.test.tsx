import { renderHook } from '@testing-library/react';
import type { ReactNode } from 'react';
import { describe, expect, it } from 'vitest';
import { QueryClientProvider } from '@tanstack/react-query';

import { useBrowserAccessMissing } from '@/features/usage/browserAccess';
import type { Permissions } from '@/ipc/generated/Permissions';
import { queryKeys } from '@/ipc/queryKeys';
import { GRANTED_PERMISSIONS, MISSING_BROWSER_PERMISSIONS } from '@/test/ipc';
import { createTestQueryClient } from '@/test/render';

function browserAccessMissing(permissions: Permissions | undefined): boolean {
  const client = createTestQueryClient();
  if (permissions !== undefined) client.setQueryData(queryKeys.permissions, permissions);
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
  return renderHook(() => useBrowserAccessMissing(), { wrapper }).result.current;
}

describe('useBrowserAccessMissing', () => {
  it('is false while permissions are unknown or all granted', () => {
    expect(browserAccessMissing(undefined)).toBe(false);
    expect(browserAccessMissing(GRANTED_PERMISSIONS)).toBe(false);
  });

  it('is true when an installed browser was denied or never asked', () => {
    expect(browserAccessMissing(MISSING_BROWSER_PERMISSIONS)).toBe(true);
    expect(
      browserAccessMissing({
        accessibility: 'granted',
        browsers: [{ browser: 'brave', installed: true, permission: 'notDetermined' }],
      }),
    ).toBe(true);
  });

  it('ignores browsers that are not installed or cannot be checked', () => {
    expect(
      browserAccessMissing({
        accessibility: 'granted',
        browsers: [
          { browser: 'edge', installed: false, permission: 'denied' },
          { browser: 'firefox', installed: true, permission: 'unsupported' },
        ],
      }),
    ).toBe(false);
  });
});
