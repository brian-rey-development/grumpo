import { SectionRow } from '@/components/Section/SectionRow';
import { SelectField, type SelectOption } from '@/components/SelectField/SelectField';
import { settingsCopy } from '@/features/settings/copy';
import { RetentionSheet } from '@/features/settings/RetentionSheet';
import { RETENTION_OPTIONS_DAYS, retentionLabel } from '@/features/settings/settingsFormat';
import { useRetentionChange } from '@/features/settings/useRetentionChange';
import type { Settings } from '@/ipc/generated/Settings';

const copy = settingsCopy.history;

function retentionOptions(current: number): SelectOption[] {
  const days = [...RETENTION_OPTIONS_DAYS];
  if (!days.includes(current)) days.push(current);
  return days
    .sort((a, b) => a - b)
    .map((value) => ({
      value: value.toString(),
      label: retentionLabel(value),
    }));
}

interface RetentionRowProps {
  settings: Settings;
  onFailed: (message: string) => void;
}

export function RetentionRow({ settings, onFailed }: RetentionRowProps) {
  const change = useRetentionChange(settings, onFailed);

  return (
    <>
      <SectionRow label={copy.keepFor} detail={copy.keepForDetail}>
        <SelectField
          label={copy.keepFor}
          options={retentionOptions(change.saved)}
          value={change.shown.toString()}
          disabled={change.pending && !change.confirming}
          onChange={change.choose}
        />
      </SectionRow>
      <RetentionSheet
        open={change.confirming}
        days={change.shown}
        pending={change.pending}
        failure={change.sheetFailure}
        onConfirm={change.confirm}
        onCancel={change.cancel}
      />
    </>
  );
}
