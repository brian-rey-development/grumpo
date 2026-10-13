import { useCallback } from 'react';

import { Dialog } from '@/components/Dialog/Dialog';
import { Mascot } from '@/components/Mascot/Mascot';
import { unlockCopy } from '@/features/unlock/copy';
import { FailedView, LoadingView, UnavailableView } from '@/features/unlock/FallbackViews';
import { PhaseStage } from '@/features/unlock/PhaseStage';
import { PhaseView } from '@/features/unlock/PhaseView';
import { resolveSheetView, type SheetView, sheetViewKey } from '@/features/unlock/phase';
import type { UnlockReason } from '@/features/unlock/types';
import { useUnlockActions } from '@/features/unlock/useUnlockActions';
import { useFocusStatus } from '@/ipc/queries';
import { useNavigation } from '@/layout/navigation';

function titleFor(view: SheetView): string {
  const titles = unlockCopy.titles;
  if (view.kind !== 'phase') return titles[view.kind];
  return titles[view.unlock.phase.kind];
}

const PHASE_POSES = {
  locked: 'shield',
  coolingDown: 'hourglass',
  ready: 'lock',
  unlocked: 'zen',
} as const;

function heroFor(view: SheetView) {
  if (view.kind !== 'phase') return undefined;
  return <Mascot pose={PHASE_POSES[view.unlock.phase.kind]} size="empty" />;
}

function announcementFor(view: SheetView): string {
  return view.kind === 'phase' ? unlockCopy.announcements[view.unlock.phase.kind] : '';
}

interface UnlockSheetProps {
  open: boolean;
  reason: UnlockReason;
  /** True when the last confirmation ran out unused. */
  lapsed: boolean;
  onClose: () => void;
}

export function UnlockSheet({ open, reason, lapsed, onClose }: UnlockSheetProps) {
  const query = useFocusStatus();
  const { navigate } = useNavigation();
  const view = resolveSheetView(query.data, query.error);
  const actions = useUnlockActions();
  const { clear } = actions;

  const close = useCallback(() => {
    clear();
    onClose();
  }, [clear, onClose]);
  const shared = { actions, reason, onClose: close };

  function openSettings(): void {
    close();
    navigate('settings');
  }

  return (
    <Dialog
      open={open}
      title={titleFor(view)}
      hero={heroFor(view)}
      dismissible={!actions.pending}
      onClose={close}
    >
      <PhaseStage phaseKey={sheetViewKey(view)}>
        {view.kind === 'loading' ? <LoadingView /> : null}
        {view.kind === 'failed' ? (
          <FailedView failure={view.failure} onRetry={() => void query.refetch()} onClose={close} />
        ) : null}
        {view.kind === 'unavailable' ? (
          <UnavailableView reason={view.reason} onOpenSettings={openSettings} onClose={close} />
        ) : null}
        {view.kind === 'phase' ? (
          <PhaseView unlock={view.unlock} lapsed={lapsed} {...shared} />
        ) : null}
      </PhaseStage>
      <div className="sr-only" role="status">
        {announcementFor(view)}
      </div>
    </Dialog>
  );
}
