import { LoadFailure } from '@/components/LoadFailure/LoadFailure';
import { Section } from '@/components/Section/Section';
import { Skeleton, SKELETON_WIDTH } from '@/components/Skeleton/Skeleton';
import { Toast } from '@/components/Toast/Toast';
import { useToast } from '@/components/Toast/useToast';
import { commonCopy } from '@/copy/common';
import { describeFailure } from '@/copy/failures';
import { GeneralSection } from '@/features/settings/GeneralSection';
import { HistorySection } from '@/features/settings/HistorySection';
import { PermissionsSection } from '@/features/settings/PermissionsSection';
import { ServiceSection } from '@/features/settings/ServiceSection';
import { settingsCopy } from '@/features/settings/copy';
import { useSettings } from '@/ipc/queries';

import styles from './Settings.module.css';

export function SettingsScreen() {
  const settings = useSettings();
  const { toast, show, dismiss } = useToast();

  return (
    <div className={styles.screen}>
      <ServiceSection />
      <PermissionsSection />
      {settings.data ? (
        <>
          <GeneralSection settings={settings.data} />
          <HistorySection settings={settings.data} onToast={show} />
        </>
      ) : (
        <Section title={settingsCopy.general.title}>
          {settings.isError ? (
            <LoadFailure
              variant="inline"
              title={settingsCopy.loadFailed}
              description={describeFailure(settings.error, commonCopy.tryAgainSoon)}
              retryLabel={commonCopy.retry}
              onRetry={() => {
                void settings.refetch();
              }}
            />
          ) : (
            <Skeleton width={SKELETON_WIDTH.short} />
          )}
        </Section>
      )}
      <Toast toast={toast} dismissLabel={commonCopy.dismiss} onDismiss={dismiss} />
    </div>
  );
}
