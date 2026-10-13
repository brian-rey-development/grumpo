import type { ReactElement, ReactNode } from 'react';
import { vi } from 'vitest';

import { UnlockProvider } from '@/features/unlock/UnlockProvider';
import { AppShell } from '@/layout/AppShell';
import { type Screen, useNavigation } from '@/layout/navigation';
import { renderWithProviders } from '@/test/render';

/** Monday 12 October 2026, mid afternoon. */
export const NOW = new Date(2026, 9, 12, 15, 30);

export function freezeClock(): void {
  vi.useFakeTimers({ toFake: ['Date'] });
  vi.setSystemTime(NOW);
}

function OnlyOn({ screen, children }: { screen: Screen; children: ReactNode }) {
  return useNavigation().screen === screen ? children : null;
}

/** Mounts a screen in the real shell and drops it when the user navigates elsewhere, like the app. */
export function renderInShell(element: ReactElement, screen: Screen = 'today') {
  return renderWithProviders(
    <UnlockProvider>
      <AppShell>
        <OnlyOn screen={screen}>{element}</OnlyOn>
      </AppShell>
    </UnlockProvider>,
    { screen },
  );
}
