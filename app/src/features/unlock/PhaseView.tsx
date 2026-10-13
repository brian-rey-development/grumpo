import { CoolingDownView } from '@/features/unlock/CoolingDownView';
import { LockedView } from '@/features/unlock/LockedView';
import { ReadyView } from '@/features/unlock/ReadyView';
import { UnlockedView } from '@/features/unlock/UnlockedView';
import type { PhaseViewProps } from '@/features/unlock/viewProps';
import type { UnlockView } from '@/ipc/generated/UnlockView';

interface PhaseViewSwitchProps extends PhaseViewProps {
  unlock: UnlockView;
  /** True when the last confirmation ran out unused. */
  lapsed: boolean;
}

export function PhaseView({ unlock, lapsed, ...shared }: PhaseViewSwitchProps) {
  const { phase, nextCooldownMinutes } = unlock;
  switch (phase.kind) {
    case 'locked':
      return <LockedView cooldownMinutes={nextCooldownMinutes} lapsed={lapsed} {...shared} />;
    case 'coolingDown':
      return <CoolingDownView readyAt={phase.readyAt} minutes={phase.minutes} {...shared} />;
    case 'ready':
      return <ReadyView unlock={unlock} expiresAt={phase.expiresAt} {...shared} />;
    case 'unlocked':
      return <UnlockedView until={phase.until} {...shared} />;
  }
}
