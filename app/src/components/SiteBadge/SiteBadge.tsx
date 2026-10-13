import { Globe } from 'lucide-react';

import styles from './SiteBadge.module.css';

const GLYPH_SIZE = 20;

export function SiteBadge() {
  return (
    <span className={styles.badge} data-testid="site-badge" aria-hidden="true">
      <Globe size={GLYPH_SIZE} />
    </span>
  );
}
