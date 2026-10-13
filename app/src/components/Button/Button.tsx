import type { ButtonHTMLAttributes, MouseEvent, ReactNode, Ref } from 'react';

import { cx } from '@/lib/classNames';

import styles from './Button.module.css';

type ButtonVariant = 'primary' | 'secondary' | 'inverse' | 'plain' | 'destructive';
type ButtonSize = 'regular' | 'small';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  /** Keeps focus and the width, and ignores clicks, so Escape and Return still reach the page. */
  loading?: boolean;
  icon?: ReactNode;
  ref?: Ref<HTMLButtonElement>;
}

export function Button({
  variant = 'secondary',
  size = 'regular',
  loading = false,
  icon,
  disabled,
  type = 'button',
  className,
  children,
  onClick,
  ...rest
}: ButtonProps) {
  const locked = disabled === true || loading;
  const classes = cx(
    styles.button,
    styles[variant],
    styles[size],
    locked && styles.locked,
    className,
  );

  function handleClick(event: MouseEvent<HTMLButtonElement>): void {
    if (loading) {
      event.preventDefault();
      return;
    }
    onClick?.(event);
  }

  return (
    <button
      {...rest}
      type={type}
      className={classes}
      disabled={disabled === true}
      aria-disabled={loading || undefined}
      aria-busy={loading || undefined}
      onClick={handleClick}
    >
      {loading ? <span className={styles.spinner} aria-hidden="true" /> : null}
      <span className={styles.content} data-loading={loading || undefined}>
        {icon}
        {children === undefined ? null : <span className={styles.label}>{children}</span>}
      </span>
    </button>
  );
}
