import { Check, Minus } from 'lucide-react';

import { Section } from '@/components/Section/Section';
import { ExplainedRow } from '@/features/explain/ExplainedRow';
import { StatusBadge } from '@/features/explain/StatusBadge';
import { settingsCopy } from '@/features/settings/copy';
import type { Permissions } from '@/ipc/generated/Permissions';
import type { PermissionStatus } from '@/ipc/generated/PermissionStatus';
import { useFocusStatus, usePermissions } from '@/ipc/queries';

import { onboardingCopy } from '../copy';

import styles from '../Step.module.css';

const copy = onboardingCopy.done;
const ICON_SIZE = 18;
const ICON_STROKE = 2.5;

interface Words {
  done: string;
  pending: string;
}

interface Item {
  key: string;
  label: string;
  done: boolean;
  status: string;
}

function itemOf(key: string, label: string, done: boolean, words: Words): Item {
  return { key, label, done, status: done ? words.done : words.pending };
}

function isGranted(status: PermissionStatus): boolean {
  return status === 'granted' || status === 'unsupported';
}

function permissionItems(permissions: Permissions): Item[] {
  const accessibility = itemOf(
    'accessibility',
    settingsCopy.permissions.accessibility,
    isGranted(permissions.accessibility),
    copy.accessibility,
  );
  const browsers = permissions.browsers
    .filter((entry) => entry.installed && entry.permission !== 'unsupported')
    .map((entry) =>
      itemOf(
        entry.browser,
        settingsCopy.browsers[entry.browser],
        entry.permission === 'granted',
        copy.browser,
      ),
    );
  return [accessibility, ...browsers];
}

function Mark({ done }: { done: boolean }) {
  const Icon = done ? Check : Minus;

  return (
    <span className={styles.mark} data-done={done} aria-hidden="true">
      <Icon size={ICON_SIZE} strokeWidth={ICON_STROKE} />
    </span>
  );
}

/** What the person ended up with, read from the live state, so a skipped step shows as plainly as a finished one. */
export function RecapList() {
  const status = useFocusStatus().data;
  const permissions = usePermissions().data;
  if (status === undefined || permissions === undefined) return null;
  const service = itemOf(
    'service',
    onboardingCopy.service.label,
    status.kind === 'connected',
    copy.service,
  );

  return (
    <div className={styles.recap}>
      <Section as="list" title={copy.recapTitle}>
        {[service, ...permissionItems(permissions)].map((item) => (
          <ExplainedRow
            key={item.key}
            label={item.label}
            badge={<StatusBadge tone="quiet" label={item.status} />}
            detail={item.done ? undefined : copy.laterNote}
            trailing={<Mark done={item.done} />}
          />
        ))}
      </Section>
    </div>
  );
}
