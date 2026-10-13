import { motion } from 'motion/react';

import { spring, useMotionTransition } from '@/design/motion';

import styles from './Toggle.module.css';

interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label: string;
  disabled?: boolean;
}

export function Toggle({ checked, onChange, label, disabled = false }: ToggleProps) {
  const transition = useMotionTransition(spring.snappy);

  return (
    <button
      type="button"
      role="switch"
      className={styles.toggle}
      aria-checked={checked}
      aria-label={label}
      disabled={disabled}
      onClick={() => {
        onChange(!checked);
      }}
    >
      <motion.span className={styles.thumb} layout transition={transition} />
    </button>
  );
}
