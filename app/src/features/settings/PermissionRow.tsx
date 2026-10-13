import { Button } from '@/components/Button/Button';
import { commonCopy } from '@/copy/common';
import { describeFailure } from '@/copy/failures';
import { settingsCopy } from '@/features/settings/copy';
import { type Explanation, explainCopy } from '@/features/explain/copy';
import { ExplainedRow } from '@/features/explain/ExplainedRow';
import { SettingsPath } from '@/features/explain/SettingsPath';
import { StatusBadge } from '@/features/explain/StatusBadge';
import { WhyDisclosure } from '@/features/explain/WhyDisclosure';
import type { PermissionKind } from '@/ipc/generated/PermissionKind';
import type { PermissionStatus } from '@/ipc/generated/PermissionStatus';
import { useOpenPermissionSettings, useRequestPermission } from '@/ipc/mutations';

import styles from './Settings.module.css';

const copy = settingsCopy.permissions;

interface PermissionRowProps {
  name: string;
  kind: PermissionKind;
  status: PermissionStatus;
}

function explanationFor(kind: PermissionKind, name: string): Explanation {
  return kind.kind === 'accessibility' ? explainCopy.accessibility : explainCopy.automation(name);
}

const BADGE_TONES = {
  granted: 'ok',
  denied: 'problem',
  notDetermined: 'quiet',
  unsupported: 'quiet',
} as const satisfies Record<Exclude<PermissionStatus, 'notRunning'>, string>;

function StateBadge({ status }: { status: PermissionStatus }) {
  if (status === 'notRunning') return null;
  return <StatusBadge tone={BADGE_TONES[status]} label={copy.states[status]} />;
}

function stateNote(status: PermissionStatus, kind: PermissionKind, name: string): string | null {
  if (status === 'notRunning') return copy.notRunning(name);
  if (status !== 'denied') return null;
  return kind.kind === 'automation' ? copy.deniedHint.browser : copy.deniedHint.accessibility;
}

/** A browser that is denied is changed in System Settings only, but Accessibility can be asked for again. */
function offersAllow(status: PermissionStatus, kind: PermissionKind): boolean {
  return status === 'notDetermined' || (status === 'denied' && kind.kind === 'accessibility');
}

function needsSystemSettingsPath(status: PermissionStatus, kind: PermissionKind): boolean {
  return kind.kind === 'accessibility' && (status === 'notDetermined' || status === 'denied');
}

/** One permission with its own pending state and failure, so a slow row never holds up the others. */
export function PermissionRow({ name, kind, status }: PermissionRowProps) {
  const request = useRequestPermission();
  const openSettings = useOpenPermissionSettings();
  const failure = request.error ?? openSettings.error;
  const busy = request.isPending || openSettings.isPending;
  const explanation = explanationFor(kind, name);
  const note = stateNote(status, kind, name);

  function allow(): void {
    openSettings.reset();
    request.mutate(kind);
  }

  function openSystemSettings(): void {
    request.reset();
    openSettings.mutate(kind);
  }

  return (
    <ExplainedRow
      label={name}
      badge={<StateBadge status={status} />}
      detail={explanation.purpose}
      trailing={
        <>
          {offersAllow(status, kind) ? (
            <Button loading={request.isPending} disabled={busy} onClick={allow}>
              {copy.allow}
            </Button>
          ) : null}
          {status === 'denied' ? (
            <Button loading={openSettings.isPending} disabled={busy} onClick={openSystemSettings}>
              {copy.openSettings}
            </Button>
          ) : null}
        </>
      }
    >
      {note === null ? null : <span className={styles.rowNote}>{note}</span>}
      {needsSystemSettingsPath(status, kind) ? (
        <SettingsPath steps={explainCopy.accessibilityPath} />
      ) : null}
      {failure === null ? null : (
        <span className={styles.rowError} role="alert">
          {`${copy.actionFailed(name)} ${describeFailure(failure, commonCopy.tryAgainSoon)}`}
        </span>
      )}
      <WhyDisclosure explanation={explanation} />
    </ExplainedRow>
  );
}
