import { Plus, X } from 'lucide-react';
import { useState } from 'react';

import { Button } from '@/components/Button/Button';
import { EmptyState } from '@/components/EmptyState/EmptyState';
import { IconButton } from '@/components/IconButton/IconButton';
import { Section } from '@/components/Section/Section';
import { SectionRow } from '@/components/Section/SectionRow';
import { AppBadge } from '@/features/rules/AppBadge';
import { AppPickerSheet } from '@/features/rules/AppPickerSheet';
import { rulesCopy } from '@/features/rules/copy';
import type { EditorProps } from '@/features/rules/editorProps';
import { addApp, removeApp, RULE_CAPS } from '@/features/rules/rulesModel';

const ICON_SIZE = 14;

export function AppsEditor({ rules, edit, readOnly }: EditorProps) {
  const copy = rulesCopy.apps;
  const [picking, setPicking] = useState(false);
  const apps = rules.blockedApps;
  const atCap = apps.length >= RULE_CAPS.apps;
  const description = atCap ? copy.capReached(RULE_CAPS.apps) : copy.description;
  const addButton = readOnly ? undefined : (
    <Button
      size="small"
      disabled={atCap}
      icon={<Plus size={ICON_SIZE} aria-hidden="true" />}
      onClick={() => {
        setPicking(true);
      }}
    >
      {copy.add}
    </Button>
  );

  const head = {
    title: copy.title,
    description,
    action: addButton,
  };

  return (
    <>
      {apps.length === 0 ? (
        <Section {...head}>
          <EmptyState title={copy.emptyTitle} description={copy.emptyDescription} />
        </Section>
      ) : (
        <Section {...head} as="list">
          {apps.map((app) => (
            <SectionRow key={app.bundleId} leading={<AppBadge app={app} />} label={app.name}>
              {readOnly ? null : (
                <IconButton
                  label={copy.remove(app.name)}
                  icon={<X size={ICON_SIZE} aria-hidden="true" />}
                  onClick={() => {
                    edit((current) => removeApp(current, app.bundleId));
                  }}
                />
              )}
            </SectionRow>
          ))}
        </Section>
      )}
      <AppPickerSheet
        open={picking}
        excluded={new Set(apps.map((app) => app.bundleId))}
        onClose={() => {
          setPicking(false);
        }}
        onPick={(app) => {
          edit((current) => addApp(current, app));
        }}
      />
    </>
  );
}
