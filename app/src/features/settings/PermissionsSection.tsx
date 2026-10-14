import { useCallback } from 'react';

import { LoadFailure } from '@/components/LoadFailure/LoadFailure';
import { Section } from '@/components/Section/Section';
import { Skeleton, SKELETON_WIDTH } from '@/components/Skeleton/Skeleton';
import { commonCopy } from '@/copy/common';
import { describeFailure } from '@/copy/failures';
import { settingsCopy } from '@/features/settings/copy';
import { FirefoxRow } from '@/features/settings/FirefoxRow';
import { PermissionRow } from '@/features/settings/PermissionRow';
import { useRefreshOnFocus } from '@/features/settings/useRefreshOnFocus';
import type { PermissionKind } from '@/ipc/generated/PermissionKind';
import type { Permissions } from '@/ipc/generated/Permissions';
import { hasAccessToAllow, usePermissions } from '@/ipc/queries';

const copy = settingsCopy.permissions;

interface Row {
  key: string;
  name: string;
  kind: PermissionKind;
  status: Permissions['accessibility'];
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

function hasFirefox(permissions: Permissions): boolean {
  return permissions.browsers.some((entry) => entry.browser === 'firefox' && entry.installed);
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
  variant?: 'section' | 'bare';
}

export function PermissionsSection({ variant = 'section' }: PermissionsSectionProps) {
  const query = usePermissions({ followPending: true });
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
      {hasFirefox(query.data) ? <FirefoxRow accessibility={query.data.accessibility} /> : null}
    </Section>
  );
}
