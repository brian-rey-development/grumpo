import type { CSSProperties } from 'react';

import { cx } from '@/lib/classNames';

import styles from './Skeleton.module.css';

export const SKELETON_WIDTH = { full: '100%', long: '70%', short: '60%' } as const;

interface SkeletonProps {
  width?: CSSProperties['width'];
  height?: CSSProperties['height'];
  radius?: CSSProperties['borderRadius'];
  className?: string | undefined;
}

export function Skeleton({
  width = SKELETON_WIDTH.full,
  height = 'var(--leading-body)',
  radius,
  className,
}: SkeletonProps) {
  const classes = cx(styles.skeleton, className);

  return (
    <span className={classes} style={{ width, height, borderRadius: radius }} aria-hidden="true" />
  );
}
