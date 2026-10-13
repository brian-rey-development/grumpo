import { motion } from 'motion/react';

import desk from '@/assets/brand/grumpo-desk.png';
import lock from '@/assets/brand/grumpo-lock.png';
import shield from '@/assets/brand/grumpo-shield.png';
import crossed from '@/assets/brand/stickers/arms-crossed.png';
import approve from '@/assets/brand/stickers/approve.png';
import wave from '@/assets/brand/stickers/wave.png';
import { spring, useMotionTransition } from '@/design/motion';

import type { PoseName } from './steps';

import styles from './Step.module.css';

const SOURCES: Record<PoseName, string> = { approve, crossed, desk, lock, shield, wave };
const SETTLE_FROM = { opacity: 0, scale: 0.92 } as const;
const SETTLE_TO = { opacity: 1, scale: 1 } as const;

export function preloadPose(name: PoseName): void {
  const image = new Image();
  image.src = SOURCES[name];
}

/** Every pose sits in the same fixed box, so the heading below never moves from one step to the next. */
export function Pose({ name }: { name: PoseName }) {
  const settle = useMotionTransition(spring.gentle);

  return (
    <div className={styles.poseBox}>
      <motion.img
        className={styles.pose}
        src={SOURCES[name]}
        alt=""
        initial={SETTLE_FROM}
        animate={SETTLE_TO}
        transition={settle}
      />
    </div>
  );
}
