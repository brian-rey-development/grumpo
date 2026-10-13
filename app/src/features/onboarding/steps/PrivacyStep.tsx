import { AppWindow, EyeOff, FileText, Globe, Keyboard, type LucideIcon } from 'lucide-react';

import { onboardingCopy } from '../copy';
import { Point } from '../Point';
import { Pose } from '../Pose';
import { STEP_POSES } from '../steps';
import { StepFrame } from '../StepFrame';

import styles from '../Step.module.css';

const copy = onboardingCopy.private;

interface ListProps {
  title: string;
  items: readonly string[];
  icons: readonly LucideIcon[];
}

function Column({ title, items, icons }: ListProps) {
  return (
    <section>
      <h2 className={styles.columnTitle}>{title}</h2>
      <ul className={styles.points}>
        {items.map((item, index) => (
          <Point key={item} icon={icons[index] ?? EyeOff} title={item} />
        ))}
      </ul>
    </section>
  );
}

export function PrivacyStep() {
  return (
    <StepFrame hero={<Pose name={STEP_POSES.private} />} title={copy.title} lede={copy.lede}>
      <div className={styles.pair}>
        <Column title={copy.recordsTitle} items={copy.records} icons={[AppWindow, Globe]} />
        <Column title={copy.neverTitle} items={copy.never} icons={[FileText, Keyboard, EyeOff]} />
      </div>
    </StepFrame>
  );
}
