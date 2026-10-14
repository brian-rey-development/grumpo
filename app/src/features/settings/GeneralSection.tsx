import { useState } from 'react';

import { Button } from '@/components/Button/Button';
import { Section } from '@/components/Section/Section';
import { SectionRow } from '@/components/Section/SectionRow';
import { Toggle } from '@/components/Toggle/Toggle';
import { commonCopy } from '@/copy/common';
import { describeFailure } from '@/copy/failures';
import { settingsCopy } from '@/features/settings/copy';
import type { Settings } from '@/ipc/generated/Settings';
import { useResetOnboarding, useUpdateSettings } from '@/ipc/mutations';

import styles from './Settings.module.css';

const copy = settingsCopy.general;

export function GeneralSection({ settings }: { settings: Settings }) {
  const update = useUpdateSettings();
  const reset = useResetOnboarding();
  const [failed, setFailed] = useState<string | null>(null);

  function save(changes: Partial<Settings>, failure: string): void {
    setFailed(null);
    update.mutate(
      { ...settings, ...changes },
      {
        onError: (error) => {
          setFailed(`${failure} ${describeFailure(error, commonCopy.tryAgainSoon)}`);
        },
      },
    );
  }

  function showWelcome(): void {
    setFailed(null);
    reset.mutate(undefined, {
      onError: (failure) => {
        setFailed(`${copy.welcomeFailed} ${describeFailure(failure, commonCopy.tryAgainSoon)}`);
      },
    });
  }

  return (
    <div className={styles.group}>
      <Section title={copy.title} as="list">
        <SectionRow label={copy.launchAtLogin} detail={copy.launchAtLoginDetail}>
          <Toggle
            label={copy.launchAtLogin}
            checked={settings.launchAtLogin}
            disabled={update.isPending}
            onChange={(launchAtLogin) => {
              save({ launchAtLogin }, copy.launchAtLoginFailed);
            }}
          />
        </SectionRow>
        <SectionRow label={copy.autoUpdate} detail={copy.autoUpdateDetail}>
          <Toggle
            label={copy.autoUpdate}
            checked={settings.autoUpdate}
            disabled={update.isPending}
            onChange={(autoUpdate) => {
              save({ autoUpdate }, copy.autoUpdateFailed);
            }}
          />
        </SectionRow>
        <SectionRow label={copy.welcome} detail={copy.welcomeDetail}>
          <Button loading={reset.isPending} onClick={showWelcome}>
            {copy.showWelcome}
          </Button>
        </SectionRow>
      </Section>
      {failed === null ? null : (
        <p className={styles.error} role="alert">
          {failed}
        </p>
      )}
    </div>
  );
}
