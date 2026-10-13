import armsCrossedLarge from '@/assets/brand/grumpo-arms-crossed.png';
import pushLarge from '@/assets/brand/grumpo-push.png';
import approve from '@/assets/brand/stickers/approve.png';
import armsCrossed from '@/assets/brand/stickers/arms-crossed.png';
import hourglass from '@/assets/brand/stickers/hourglass.png';
import laptop from '@/assets/brand/stickers/laptop.png';
import lock from '@/assets/brand/stickers/lock.png';
import shield from '@/assets/brand/stickers/shield.png';
import sleep from '@/assets/brand/stickers/sleep.png';
import stop from '@/assets/brand/stickers/stop.png';
import trash from '@/assets/brand/stickers/trash.png';
import zen from '@/assets/brand/stickers/zen.png';
import { cx } from '@/lib/classNames';

import styles from './Mascot.module.css';

const POSES = {
  approve,
  armsCrossed,
  armsCrossedLarge,
  hourglass,
  laptop,
  lock,
  pushLarge,
  shield,
  sleep,
  stop,
  trash,
  zen,
} as const;

export type MascotPose = keyof typeof POSES;

interface MascotProps {
  pose: MascotPose;
  size: 'badge' | 'empty' | 'takeover';
  className?: string | undefined;
}

/** Starts loading poses ahead of time, so swapping one in never shows an empty frame. */
export function preloadMascots(poses: readonly MascotPose[]): void {
  for (const pose of poses) {
    const image = new Image();
    image.src = POSES[pose];
    if ('decode' in image) void image.decode().catch(() => undefined);
  }
}

export function Mascot({ pose, size, className }: MascotProps) {
  return (
    <img
      className={cx(styles.mascot, styles[size], className)}
      src={POSES[pose]}
      alt=""
      decoding="async"
      draggable={false}
    />
  );
}
