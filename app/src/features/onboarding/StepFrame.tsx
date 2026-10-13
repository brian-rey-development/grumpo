import { type ReactNode, useEffect, useRef } from 'react';

import { cx } from '@/lib/classNames';

import styles from './Step.module.css';

interface StepFrameProps {
  /** The picture above the heading. */
  hero?: ReactNode;
  title: ReactNode;
  lede?: string;
  centered?: boolean;
  /** A short header row for steps with a list, so the list fits the default window. */
  compact?: boolean;
  children?: ReactNode;
}

/** Every step is one heading, one line of context and its content, and the heading takes focus on arrival. */
export function StepFrame({
  hero,
  title,
  lede,
  centered = false,
  compact = false,
  children,
}: StepFrameProps) {
  const headingRef = useRef<HTMLHeadingElement>(null);

  useEffect(() => {
    headingRef.current?.focus({ preventScroll: true });
  }, []);

  return (
    <section className={cx(styles.step, compact && styles.compact)}>
      {hero}
      <h1 ref={headingRef} className={styles.heading} tabIndex={-1}>
        {title}
      </h1>
      {lede === undefined ? null : (
        <p className={styles.lede} data-selectable>
          {lede}
        </p>
      )}
      {children === undefined ? null : (
        <div className={cx(styles.content, centered && styles.centered)}>{children}</div>
      )}
    </section>
  );
}
