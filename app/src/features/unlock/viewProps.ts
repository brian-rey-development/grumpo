import type { UnlockActions } from '@/features/unlock/useUnlockActions';
import type { UnlockReason } from '@/features/unlock/types';

export interface PhaseViewProps {
  actions: UnlockActions;
  reason: UnlockReason;
  onClose: () => void;
}
