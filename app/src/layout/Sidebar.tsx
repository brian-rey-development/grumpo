import { CalendarDays, type LucideIcon, Settings2, ShieldCheck, Sun } from 'lucide-react';
import { motion } from 'motion/react';
import type { ReactNode } from 'react';

import { LogoTile } from '@/components/LogoTile/LogoTile';
import { spring, useMotionTransition } from '@/design/motion';
import { layoutCopy } from '@/layout/copy';
import { SCREEN_LABELS, SCREENS, type Screen, useNavigation } from '@/layout/navigation';

import styles from './Sidebar.module.css';

const ICON_SIZE = 20;

const NAV_ICONS: Record<Screen, LucideIcon> = {
  today: Sun,
  week: CalendarDays,
  rules: ShieldCheck,
  settings: Settings2,
};

interface SidebarProps {
  footer?: ReactNode;
  /** Screens that hold work which is not saved yet, each marked with a small dot. */
  unsaved?: readonly Screen[];
}

export function Sidebar({ footer, unsaved = [] }: SidebarProps) {
  const { screen, navigate } = useNavigation();
  const indicatorTransition = useMotionTransition(spring.snappy);

  return (
    <aside className={styles.sidebar} aria-label={layoutCopy.productName}>
      <div className={styles.titlebar} data-tauri-drag-region />
      <div className={styles.brand}>
        <LogoTile />
        <span className={styles.name}>{layoutCopy.productName}</span>
      </div>
      <nav className={styles.nav} aria-label={layoutCopy.navLabel}>
        <ul className={styles.list}>
          {SCREENS.map((target) => {
            const active = target === screen;
            const Icon = NAV_ICONS[target];
            return (
              <li key={target}>
                <button
                  type="button"
                  className={styles.item}
                  aria-current={active ? 'page' : undefined}
                  onClick={() => {
                    navigate(target);
                  }}
                >
                  {active ? (
                    <motion.span
                      layoutId="sidebar-active"
                      className={styles.indicator}
                      transition={indicatorTransition}
                    />
                  ) : null}
                  <Icon className={styles.icon} size={ICON_SIZE} aria-hidden="true" />
                  <span className={styles.label}>{SCREEN_LABELS[target]}</span>
                  {unsaved.includes(target) ? (
                    <>
                      <span className={styles.dot} aria-hidden="true" data-testid="unsaved-dot" />
                      <span className="sr-only">{`, ${layoutCopy.unsavedChanges}`}</span>
                    </>
                  ) : null}
                </button>
              </li>
            );
          })}
        </ul>
      </nav>
      {footer === undefined ? null : <div className={styles.footer}>{footer}</div>}
    </aside>
  );
}
