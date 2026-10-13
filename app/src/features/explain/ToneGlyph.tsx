import type { LucideIcon } from 'lucide-react';

import styles from './ToneGlyph.module.css';

export type Tone = 'focus' | 'limit' | 'unlocked';

const ICON_SIZE = 18;

/** A bare glyph that carries a color, with no tile or circle behind it. */
export function ToneGlyph({ tone, icon: Icon }: { tone: Tone; icon: LucideIcon }) {
  return (
    <span className={styles.glyph} data-tone={tone} aria-hidden="true">
      <Icon size={ICON_SIZE} />
    </span>
  );
}
