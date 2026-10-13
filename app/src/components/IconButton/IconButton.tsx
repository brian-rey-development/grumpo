import type { ButtonHTMLAttributes, ReactNode, Ref } from 'react';

import { cx } from '@/lib/classNames';

import styles from './IconButton.module.css';

type IconButtonSize = 'regular' | 'small';

interface IconButtonProps extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'children'> {
  label: string;
  icon: ReactNode;
  size?: IconButtonSize;
  ref?: Ref<HTMLButtonElement>;
}

export function IconButton({
  label,
  icon,
  size = 'regular',
  type = 'button',
  className,
  ...rest
}: IconButtonProps) {
  const classes = cx(styles.button, styles[size], className);

  return (
    <button {...rest} type={type} className={classes} aria-label={label} title={label}>
      {icon}
    </button>
  );
}
