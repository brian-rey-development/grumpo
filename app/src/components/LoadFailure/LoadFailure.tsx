import { Button } from '@/components/Button/Button';
import { EmptyState } from '@/components/EmptyState/EmptyState';

import styles from './LoadFailure.module.css';

interface LoadFailureProps {
  title: string;
  description?: string | undefined;
  retryLabel: string;
  onRetry: () => void;
  /** `page` fills a screen, and `inline` sits inside a section next to other rows. */
  variant?: 'page' | 'inline';
}

/** One plain line about what failed, with a way to try again, on the left edge of the page. */
export function LoadFailure({
  title,
  description,
  retryLabel,
  onRetry,
  variant = 'page',
}: LoadFailureProps) {
  const retry = <Button onClick={onRetry}>{retryLabel}</Button>;
  if (variant === 'page') {
    return (
      <EmptyState
        variant="page"
        role="alert"
        title={title}
        description={description}
        action={retry}
      />
    );
  }

  return (
    <div className={styles.inline}>
      <p role="alert">{title}</p>
      {description === undefined ? null : <p className={styles.detail}>{description}</p>}
      {retry}
    </div>
  );
}
