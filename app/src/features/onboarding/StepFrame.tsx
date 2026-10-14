import { type ReactNode, useEffect, useRef } from 'react';

import { cx } from '@/lib/classNames';

import styles from './Step.module.css';

interface StepFrameProps {
  hero?: ReactNode;
  title: ReactNode;
  lede?: string;
  centered?: boolean;
  compact?: boolean;
  children?: ReactNode;
}

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

  const header = (
    <>
      {hero}
      <h1 ref={headingRef} className={styles.heading} tabIndex={-1}>
        {title}
      </h1>
    </>
  );

  return (
    <section className={cx(styles.step, compact && styles.compact)}>
      {compact ? <div className={styles.header}>{header}</div> : header}
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
