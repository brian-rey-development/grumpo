import { Button } from '@/components/Button/Button';
import { EmptyState } from '@/components/EmptyState/EmptyState';
import { LoadFailure } from '@/components/LoadFailure/LoadFailure';
import { Mascot } from '@/components/Mascot/Mascot';
import { Skeleton } from '@/components/Skeleton/Skeleton';
import { commonCopy } from '@/copy/common';
import { describeFailure } from '@/copy/failures';
import { rulesCopy } from '@/features/rules/copy';
import { RulesEditor } from '@/features/rules/RulesEditor';
import { useRulesDraftContext } from '@/features/rules/rulesDraftContext';
import { useFocusStatus } from '@/ipc/queries';
import { useNavigation } from '@/layout/navigation';

import styles from './Editors.module.css';

const SKELETON_SECTIONS = 3;

function RulesSkeleton() {
  return (
    <div className={styles.stack} role="status" aria-label={rulesCopy.screen.loading}>
      {Array.from({ length: SKELETON_SECTIONS }, (_, section) => (
        <Skeleton key={section} height="var(--size-skeleton-section)" radius="var(--radius-xl)" />
      ))}
    </div>
  );
}

export function RulesScreen() {
  const query = useFocusStatus();
  const { navigate } = useNavigation();
  const draft = useRulesDraftContext();

  if (draft.rules !== null) return <RulesEditor draft={draft} />;
  if (query.isPending) return <RulesSkeleton />;
  if (query.isError) {
    return (
      <LoadFailure
        title={rulesCopy.screen.loadFailedTitle}
        description={describeFailure(query.error, commonCopy.tryAgainSoon)}
        retryLabel={commonCopy.retry}
        onRetry={() => {
          void query.refetch();
        }}
      />
    );
  }
  return (
    <EmptyState
      visual={<Mascot pose="stop" size="empty" />}
      title={rulesCopy.unavailable.title}
      description={rulesCopy.unavailable.description}
      action={
        <Button
          onClick={() => {
            navigate('settings');
          }}
        >
          {commonCopy.openSettings}
        </Button>
      }
    />
  );
}
