import { useCallback } from 'react';

import { LoadFailure } from '@/components/LoadFailure/LoadFailure';
import { Section } from '@/components/Section/Section';
import { Skeleton, SKELETON_WIDTH } from '@/components/Skeleton/Skeleton';
import { commonCopy } from '@/copy/common';
import { describeFailure } from '@/copy/failures';
import { settingsCopy } from '@/features/settings/copy';
import { PermissionRow } from '@/features/settings/PermissionRow';
import { useRefreshOnFocus } from '@/features/settings/useRefreshOnFocus';
import type { BrowserAccess } from '@/ipc/generated/BrowserAccess';
import type { PermissionKind } from '@/ipc/generated/PermissionKind';
import type { Permissions } from '@/ipc/generated/Permissions';
import { usePermissions } from '@/ipc/queries';

const copy = settingsCopy.permissions;

interface Row {
  key: string;
  name: string;
  kind: PermissionKind;
  status: Permissions['accessibility'];
}

/** A browser that is not on this Mac, or that Grumpo reads without asking, has nothing to allow. */
function hasAccessToAllow(entry: BrowserAccess): boolean {
  return entry.installed && entry.permission !== 'unsupported';
}

function rowsOf(permissions: Permissions): Row[] {
  const browsers = permissions.browsers.filter(hasAccessToAllow).map((entry) => ({
    key: entry.browser,
    name: settingsCopy.browsers[entry.browser],
    kind: { kind: 'automation', browser: entry.browser } as const,
    status: entry.permission,
  }));
  const accessibility = {
    key: 'accessibility',
    name: copy.accessibility,
    kind: { kind: 'accessibility' } as const,
    status: permissions.accessibility,
  };
  return [accessibility, ...browsers];
}

function Placeholder({ query }: { query: ReturnType<typeof usePermissions> }) {
  if (!query.isError) return <Skeleton width={SKELETON_WIDTH.short} />;
  return (
    <LoadFailure
      variant="inline"
      title={copy.loadFailed}
      description={describeFailure(query.error, commonCopy.tryAgainSoon)}
      retryLabel={commonCopy.retry}
      onRetry={() => {
        void query.refetch();
      }}
    />
  );
}

interface PermissionsSectionProps {
  /** `bare` leaves out the title and description, for a screen that already introduces the list. */
  variant?: 'section' | 'bare';
}

export function PermissionsSection({ variant = 'section' }: PermissionsSectionProps) {
  const query = usePermissions();
  const { refetch } = query;
  const refresh = useCallback(() => {
    void refetch();
  }, [refetch]);
  useRefreshOnFocus(refresh);

  const heading = variant === 'bare' ? {} : { title: copy.title, description: copy.description };

  if (query.data === undefined) {
    return (
      <Section {...heading}>
        <Placeholder query={query} />
      </Section>
    );
  }
  return (
    <Section {...heading} as="list">
      {rowsOf(query.data).map((row) => (
        <PermissionRow key={row.key} name={row.name} kind={row.kind} status={row.status} />
      ))}
    </Section>
  );
}
