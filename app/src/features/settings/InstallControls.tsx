import type { ReactNode } from 'react';

import { Button } from '@/components/Button/Button';
import { commonCopy } from '@/copy/common';
import { describeFailure } from '@/copy/failures';
import { INSTALL_COMMAND } from '@/copy/service';
import { CopyableCode } from '@/features/settings/CopyableCode';
import { settingsCopy } from '@/features/settings/copy';
import type { InstallableReason } from '@/features/settings/serviceFix';
import type { ServiceInstall } from '@/features/settings/useServiceInstall';

import styles from './Settings.module.css';

const copy = settingsCopy.service;

interface InstallProps {
  reason: InstallableReason;
  install: ServiceInstall;
}

export function InstallButton({ reason, install }: InstallProps) {
  return (
    <Button
      variant="primary"
      className={styles.installButton}
      loading={install.pending}
      onClick={install.start}
    >
      {install.failure === null ? copy.install[reason] : commonCopy.retry}
    </Button>
  );
}

/** Always rendered, because a live region has to exist before its text changes to be announced. */
export function InstallProgress({ reason, install }: InstallProps) {
  return (
    <p className={styles.secondary} role="status">
      {install.pending ? copy.progress[reason] : ''}
    </p>
  );
}

interface InstallFallbackProps {
  install: ServiceInstall;
  intro: string;
  children?: ReactNode;
}

function TerminalCommand({ intro, children }: Omit<InstallFallbackProps, 'install'>) {
  return (
    <div className={styles.stack}>
      <p data-selectable>{intro}</p>
      <CopyableCode label={copy.commandLabel} code={INSTALL_COMMAND} />
      {children}
    </div>
  );
}

/** What follows a failed install, with the Terminal command as a disclosure, or shown outright when this build cannot install. */
export function InstallFallback({ install, intro, children }: InstallFallbackProps) {
  const { failure } = install;
  if (failure === null) return null;
  const reason = describeFailure(failure, commonCopy.tryAgainSoon);

  if (install.unavailable) {
    return <TerminalCommand intro={`${reason} ${intro}`}>{children}</TerminalCommand>;
  }
  return (
    <div className={styles.stack}>
      <p className={styles.errorText} role="alert" data-selectable>
        {reason}
      </p>
      <details className={styles.disclosure}>
        <summary className={styles.disclosureSummary}>{copy.terminalInstead}</summary>
        <TerminalCommand intro={intro}>{children}</TerminalCommand>
      </details>
    </div>
  );
}
