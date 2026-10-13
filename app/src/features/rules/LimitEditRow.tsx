import { X } from 'lucide-react';

import { IconButton } from '@/components/IconButton/IconButton';
import { SectionRow } from '@/components/Section/SectionRow';
import { TargetBadge } from '@/features/rules/AppBadge';
import { rulesCopy } from '@/features/rules/copy';
import { LimitStepper } from '@/features/rules/LimitStepper';
import type { Limit } from '@/ipc/generated/Limit';
import { formatDuration } from '@/lib/format';
import { targetName } from '@/lib/limitTarget';

const ICON_SIZE = 14;
const copy = rulesCopy.limits;

function usedText(usedSeconds: number | undefined): string | undefined {
  if (usedSeconds === undefined) return undefined;
  return usedSeconds === 0 ? copy.unused : copy.used(formatDuration(usedSeconds));
}

interface LimitEditRowProps {
  limit: Limit;
  /** Seconds used today, or undefined while the usage is not known. */
  usedSeconds: number | undefined;
  readOnly: boolean;
  onChange: (dailyMinutes: number) => void;
  onRemove: () => void;
}

export function LimitEditRow({
  limit,
  usedSeconds,
  readOnly,
  onChange,
  onRemove,
}: LimitEditRowProps) {
  const name = targetName(limit.target);

  return (
    <SectionRow
      leading={<TargetBadge target={limit.target} />}
      label={name}
      detail={usedText(usedSeconds)}
    >
      <LimitStepper
        label={copy.stepperLabel(name)}
        decreaseLabel={copy.decrease(name)}
        increaseLabel={copy.increase(name)}
        value={limit.dailyMinutes}
        disabled={readOnly}
        onChange={onChange}
      />
      {readOnly ? null : (
        <IconButton
          label={copy.remove(name)}
          icon={<X size={ICON_SIZE} aria-hidden="true" />}
          onClick={onRemove}
        />
      )}
    </SectionRow>
  );
}
