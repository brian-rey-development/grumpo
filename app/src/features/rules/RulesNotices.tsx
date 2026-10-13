import { Button } from '@/components/Button/Button';
import { Notice } from '@/components/Notice/Notice';
import { commonCopy } from '@/copy/common';
import { describeFailure } from '@/copy/failures';
import { rulesCopy } from '@/features/rules/copy';
import type { CommandFailure } from '@/ipc/errors';

const copy = rulesCopy.notices;

interface LooseningNoticeProps {
  onUnlock: () => void;
  /** False outside focus time, when only the daily limits are locked. */
  blocking: boolean;
  /** True when a locked change raises or removes a daily limit, which waiting for the block to end cannot save. */
  includesLimitChange: boolean;
}

export function LooseningNotice({ onUnlock, blocking, includesLimitChange }: LooseningNoticeProps) {
  const waitsForBlockEnd = blocking && !includesLimitChange;
  return (
    <Notice
      tone="warning"
      role="alert"
      title={copy.loosening.title}
      action={
        <Button variant="primary" onClick={onUnlock}>
          {copy.loosening.action}
        </Button>
      }
    >
      {waitsForBlockEnd ? copy.loosening.body : copy.loosening.limitsBody}
    </Notice>
  );
}

export function SaveFailureNotice({ failure }: { failure: CommandFailure }) {
  return (
    <Notice tone="danger" role="alert" title={copy.saveFailedTitle}>
      {describeFailure(failure, commonCopy.tryAgainSoon)}
    </Notice>
  );
}

export function DraftProblemsNotice() {
  return (
    <Notice tone="danger" role="status">
      {copy.draftProblems}
    </Notice>
  );
}

interface ReadOnlyNoticeProps {
  /** True when unsaved changes are waiting, which the notice promises to keep. */
  hasEdits: boolean;
  onOpenSettings: () => void;
}

export function ReadOnlyNotice({ hasEdits, onOpenSettings }: ReadOnlyNoticeProps) {
  return (
    <Notice
      title={commonCopy.serviceNotRunning}
      action={<Button onClick={onOpenSettings}>{commonCopy.openSettings}</Button>}
    >
      {hasEdits ? `${copy.readOnly.body} ${copy.readOnly.keptEdits}` : copy.readOnly.body}
    </Notice>
  );
}
