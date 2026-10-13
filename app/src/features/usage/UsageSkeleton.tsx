import { Section } from '@/components/Section/Section';
import { Skeleton } from '@/components/Skeleton/Skeleton';

import styles from './UsageSkeleton.module.css';

const LIST_ROWS = 4;
const LABEL_WIDTH = 96;
const NUMBER_WIDTH = 180;
const LINE_WIDTH = 220;

export interface SkeletonBlock {
  title: string;
  height: string;
}

interface UsageSkeletonProps {
  loadingLabel: string;
  blocks: readonly SkeletonBlock[];
  listTitles: readonly [string, string];
  /** Reserves the height of the duty banner, for the screen that has one. */
  banner?: boolean;
}

function ListSkeleton({ title }: { title: string }) {
  return (
    <Section title={title}>
      <div className={styles.rows}>
        {Array.from({ length: LIST_ROWS }, (_, index) => (
          <div key={index} className={styles.row}>
            <Skeleton
              width="var(--size-icon-regular)"
              height="var(--size-icon-regular)"
              radius="var(--radius-sm)"
            />
            <Skeleton height="var(--size-icon-regular)" radius="var(--radius-sm)" />
          </div>
        ))}
      </div>
    </Section>
  );
}

/** Mirrors the loaded layout block for block, so the real content lands without moving anything. */
export function UsageSkeleton({
  loadingLabel,
  blocks,
  listTitles,
  banner = false,
}: UsageSkeletonProps) {
  return (
    <div className={styles.stack} role="status" aria-label={loadingLabel}>
      {banner ? <Skeleton height="var(--size-banner)" radius="var(--radius-xl)" /> : null}
      <div className={styles.hero}>
        <Skeleton width={LABEL_WIDTH} height="var(--leading-body)" />
        <Skeleton width={NUMBER_WIDTH} height="var(--leading-hero)" radius="var(--radius-md)" />
        <Skeleton width={LINE_WIDTH} height="var(--leading-body)" />
      </div>
      {blocks.map((block) => (
        <Section boxed key={block.title} title={block.title}>
          <Skeleton height={block.height} radius="var(--radius-md)" />
        </Section>
      ))}
      <div className={styles.columns}>
        {listTitles.map((title) => (
          <ListSkeleton key={title} title={title} />
        ))}
      </div>
    </div>
  );
}
