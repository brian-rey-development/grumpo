import { useState } from 'react';

import { Button } from '@/components/Button/Button';
import { Section } from '@/components/Section/Section';
import { Skeleton, SKELETON_WIDTH } from '@/components/Skeleton/Skeleton';
import { serviceCopy } from '@/copy/service';
import { explainCopy } from '@/features/explain/copy';
import { WhyDisclosure } from '@/features/explain/WhyDisclosure';
import { settingsCopy } from '@/features/settings/copy';
import {
  InstallButton,
  InstallFallback,
  InstallProgress,
} from '@/features/settings/InstallControls';
import { type InstallableReason, offersInstall } from '@/features/settings/serviceFix';
import { useServiceInstall } from '@/features/settings/useServiceInstall';
import type { FocusStatus } from '@/ipc/generated/FocusStatus';
import { useFocusStatus } from '@/ipc/queries';

import styles from './Settings.module.css';

const copy = settingsCopy.service;

function Connected() {
  return (
    <div className={styles.stack}>
      <p className={styles.headline}>{copy.running}</p>
      <p className={styles.secondary}>{copy.runningDetail}</p>
    </div>
  );
}

function CheckAgain() {
  const query = useFocusStatus();
  const [checked, setChecked] = useState(false);

  async function checkAgain(): Promise<void> {
    await query.refetch();
    setChecked(true);
  }

  return (
    <div className={styles.inline}>
      <Button loading={query.isFetching} onClick={() => void checkAgain()}>
        {copy.checkAgain}
      </Button>
      <p className={styles.secondary} role="status">
        {checked && !query.isFetching ? copy.stillDown : ''}
      </p>
    </div>
  );
}

function Installable({ reason }: { reason: InstallableReason }) {
  const install = useServiceInstall();

  return (
    <>
      {install.unavailable ? null : (
        <>
          <p className={styles.secondary}>{copy.passwordNote[reason]}</p>
          <div className={styles.inline}>
            <InstallButton reason={reason} install={install} />
            <InstallProgress reason={reason} install={install} />
          </div>
        </>
      )}
      <InstallFallback install={install} intro={serviceCopy.runCommand}>
        <CheckAgain />
      </InstallFallback>
    </>
  );
}

function Unavailable({ status }: { status: Extract<FocusStatus, { kind: 'unavailable' }> }) {
  const { reason } = status;

  return (
    <div className={styles.stack}>
      <p data-selectable>{serviceCopy.reasons[reason]}</p>
      {offersInstall(reason) ? <Installable reason={reason} /> : <CheckAgain />}
    </div>
  );
}

export function ServiceSection() {
  const query = useFocusStatus();
  const status = query.data;

  return (
    <Section title={copy.title} description={copy.description}>
      {status === undefined ? <Skeleton width={SKELETON_WIDTH.short} /> : null}
      {status?.kind === 'connected' ? <Connected /> : null}
      {status?.kind === 'unavailable' ? <Unavailable status={status} /> : null}
      {status === undefined ? null : <WhyDisclosure explanation={explainCopy.service} />}
    </Section>
  );
}
