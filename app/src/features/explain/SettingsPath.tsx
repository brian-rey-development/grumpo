import { ChevronRight } from 'lucide-react';
import { Fragment } from 'react';

import { explainCopy } from './copy';

import styles from './SettingsPath.module.css';

const ICON_SIZE = 12;

/** Where macOS sends the person to turn Accessibility on, drawn as the path they will click through. */
export function SettingsPath({ steps }: { steps: readonly string[] }) {
  return (
    <ol className={styles.path} aria-label={explainCopy.pathLabel}>
      {steps.map((step, index) => (
        <Fragment key={step}>
          {index === 0 ? null : (
            <li className={styles.separator} aria-hidden="true">
              <ChevronRight size={ICON_SIZE} />
            </li>
          )}
          <li className={styles.step}>{step}</li>
        </Fragment>
      ))}
    </ol>
  );
}
