import { useState } from 'react';

import { Button } from '@/components/Button/Button';
import { Section } from '@/components/Section/Section';
import { SectionRow } from '@/components/Section/SectionRow';
import { commonCopy } from '@/copy/common';
import { describeFailure } from '@/copy/failures';
import { DeleteDataSheet, type TodayPolicy } from '@/features/settings/DeleteDataSheet';
import { settingsCopy } from '@/features/settings/copy';
import { RetentionRow } from '@/features/settings/RetentionRow';
import { fileNameOf } from '@/features/settings/settingsFormat';
import type { FocusStatus } from '@/ipc/generated/FocusStatus';
import type { Settings } from '@/ipc/generated/Settings';
import { useFocusStatus } from '@/ipc/queries';
import { useExportData } from '@/ipc/mutations';

import styles from './Settings.module.css';

const copy = settingsCopy.history;

function todayPolicyFor(status: FocusStatus | undefined): TodayPolicy {
  if (status?.kind !== 'connected') return 'keptUnknown';
  return status.status.rules.limits.length > 0 ? 'kept' : 'deleted';
}

interface HistorySectionProps {
  settings: Settings;
  onToast: (text: string) => void;
}

export function HistorySection({ settings, onToast }: HistorySectionProps) {
  const exportData = useExportData();
  const status = useFocusStatus();
  const [deleting, setDeleting] = useState(false);
  const [failed, setFailed] = useState<string | null>(null);

  function runExport(): void {
    setFailed(null);
    exportData.mutate(undefined, {
      onSuccess: (path) => {
        onToast(copy.exported(fileNameOf(path)));
      },
      onError: (failure) => {
        setFailed(`${copy.exportFailed} ${describeFailure(failure, commonCopy.tryAgainSoon)}`);
      },
    });
  }

  return (
    <div className={styles.group}>
      <Section title={copy.title} description={copy.description} as="list">
        <RetentionRow settings={settings} onFailed={setFailed} />
        <SectionRow label={copy.export} detail={copy.exportDetail}>
          <Button loading={exportData.isPending} onClick={runExport}>
            {copy.export}
          </Button>
        </SectionRow>
        <SectionRow label={copy.delete} detail={copy.deleteDetail}>
          <Button
            variant="destructive"
            onClick={() => {
              setDeleting(true);
            }}
          >
            {copy.delete}
          </Button>
        </SectionRow>
      </Section>
      {failed === null ? null : (
        <p className={styles.error} role="alert">
          {failed}
        </p>
      )}
      <DeleteDataSheet
        open={deleting}
        today={todayPolicyFor(status.data)}
        onClose={() => {
          setDeleting(false);
        }}
        onDeleted={() => {
          setDeleting(false);
          onToast(copy.deleted);
        }}
      />
    </div>
  );
}
