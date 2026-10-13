import { createContext, useContext, useEffect } from 'react';

export const SCREENS = ['today', 'week', 'rules', 'settings'] as const;

export type Screen = (typeof SCREENS)[number];

export const SCREEN_LABELS: Record<Screen, string> = {
  today: 'Today',
  week: 'Week',
  rules: 'Rules',
  settings: 'Settings',
};

interface NavigationValue {
  screen: Screen;
  navigate: (screen: Screen) => void;
}

export const NavigationContext = createContext<NavigationValue | null>(null);

export function useNavigation(): NavigationValue {
  const value = useContext(NavigationContext);
  if (value === null) {
    throw new Error('useNavigation must be used inside a NavigationProvider');
  }
  return value;
}

type ShortcutEvent = Pick<KeyboardEvent, 'key' | 'metaKey' | 'ctrlKey' | 'altKey' | 'shiftKey'>;

function screenForShortcut(event: ShortcutEvent): Screen | null {
  if (!event.metaKey || event.ctrlKey || event.altKey || event.shiftKey) return null;
  const position = Number.parseInt(event.key, 10);
  return SCREENS[position - 1] ?? null;
}

export function useScreenShortcuts(navigate: (screen: Screen) => void, enabled = true): void {
  useEffect(() => {
    if (!enabled) return undefined;
    function onKeyDown(event: KeyboardEvent): void {
      const target = screenForShortcut(event);
      if (target === null) return;
      event.preventDefault();
      navigate(target);
    }
    window.addEventListener('keydown', onKeyDown);
    return () => {
      window.removeEventListener('keydown', onKeyDown);
    };
  }, [navigate, enabled]);
}
