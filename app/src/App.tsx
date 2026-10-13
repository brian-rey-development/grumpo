import { type QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MotionConfig } from 'motion/react';
import type { ReactElement } from 'react';

import { ModalStackProvider } from '@/components/Dialog/ModalStackProvider';
import { RevealProvider } from '@/components/Reveal/Reveal';
import { OnboardingGate } from '@/features/onboarding/OnboardingGate';
import { RulesDraftProvider } from '@/features/rules/RulesDraftProvider';
import { useRulesDirty } from '@/features/rules/rulesDraftContext';
import { RulesScreen } from '@/features/rules/RulesScreen';
import { SettingsScreen } from '@/features/settings/SettingsScreen';
import { SidebarStatus } from '@/features/status/SidebarStatus';
import { TodayScreen } from '@/features/today/TodayScreen';
import { UnlockProvider } from '@/features/unlock/UnlockProvider';
import { useUnlock } from '@/features/unlock/unlockContext';
import { WeekScreen } from '@/features/week/WeekScreen';
import { queryClient } from '@/ipc/queryClient';
import { useBackendSync } from '@/ipc/useBackendSync';
import { AppShell } from '@/layout/AppShell';
import { type Screen, useNavigation } from '@/layout/navigation';
import { NavigationProvider } from '@/layout/NavigationProvider';

const SCREEN_VIEWS: Record<Screen, () => ReactElement> = {
  today: TodayScreen,
  week: WeekScreen,
  rules: RulesScreen,
  settings: SettingsScreen,
};

const UNSAVED_RULES: readonly Screen[] = ['rules'];
const NO_UNSAVED: readonly Screen[] = [];

function AppContent() {
  const { screen, navigate } = useNavigation();
  const unlock = useUnlock();
  const rulesDirty = useRulesDirty();
  const ScreenView = SCREEN_VIEWS[screen];

  return (
    <AppShell
      unsavedScreens={rulesDirty ? UNSAVED_RULES : NO_UNSAVED}
      sidebarFooter={
        <SidebarStatus
          onOpenUnlock={() => {
            unlock.open();
          }}
          onOpenSettings={() => {
            navigate('settings');
          }}
        />
      }
    >
      <ScreenView />
    </AppShell>
  );
}

function Root() {
  useBackendSync();
  return (
    <OnboardingGate>
      <AppContent />
    </OnboardingGate>
  );
}

export function App({ client = queryClient }: { client?: QueryClient }) {
  return (
    <MotionConfig reducedMotion="user">
      <QueryClientProvider client={client}>
        <RevealProvider>
          <ModalStackProvider>
            <NavigationProvider>
              <RulesDraftProvider>
                <UnlockProvider>
                  <Root />
                </UnlockProvider>
              </RulesDraftProvider>
            </NavigationProvider>
          </ModalStackProvider>
        </RevealProvider>
      </QueryClientProvider>
    </MotionConfig>
  );
}
