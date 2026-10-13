import { type ReactNode, useMemo, useState } from 'react';

import { useIsModalOpen } from '@/components/Dialog/modalStack';
import { NavigationContext, type Screen, useScreenShortcuts } from '@/layout/navigation';

interface NavigationProviderProps {
  initialScreen?: Screen;
  children: ReactNode;
}

export function NavigationProvider({ initialScreen = 'today', children }: NavigationProviderProps) {
  const [screen, setScreen] = useState<Screen>(initialScreen);
  const modalOpen = useIsModalOpen();
  useScreenShortcuts(setScreen, !modalOpen);
  const value = useMemo(() => ({ screen, navigate: setScreen }), [screen]);

  return <NavigationContext value={value}>{children}</NavigationContext>;
}
