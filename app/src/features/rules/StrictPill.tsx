import { Lock } from 'lucide-react';

import { rulesCopy } from '@/features/rules/copy';

import styles from './StrictPill.module.css';

const ICON_SIZE = 12;
const copy = rulesCopy.strict;

/** The brand's dark pill, shown where loosening a rule needs an unlock first. */
export function StrictPill() {
  return (
    <span className={styles.pill}>
      <Lock className={styles.lock} size={ICON_SIZE} aria-hidden="true" />
      {copy.label}
      <span className="sr-only">{`, ${copy.hint}`}</span>
    </span>
  );
}
