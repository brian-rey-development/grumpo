import { Plus, X } from 'lucide-react';
import { useState } from 'react';

import { Button } from '@/components/Button/Button';
import { EmptyState } from '@/components/EmptyState/EmptyState';
import { IconButton } from '@/components/IconButton/IconButton';
import { Section } from '@/components/Section/Section';
import { SectionRow } from '@/components/Section/SectionRow';
import { AddSiteSheet } from '@/features/rules/AddSiteSheet';
import { rulesCopy } from '@/features/rules/copy';
import type { EditorProps } from '@/features/rules/editorProps';
import { addSite, removeSite, RULE_CAPS } from '@/features/rules/rulesModel';

const ICON_SIZE = 14;
const copy = rulesCopy.sites;

export function SitesEditor({ rules, edit, readOnly }: EditorProps) {
  const [adding, setAdding] = useState(false);
  const sites = rules.blockedSites;
  const atCap = sites.length >= RULE_CAPS.sites;
  const addButton = readOnly ? undefined : (
    <Button
      size="small"
      disabled={atCap}
      icon={<Plus size={ICON_SIZE} aria-hidden="true" />}
      onClick={() => {
        setAdding(true);
      }}
    >
      {copy.add}
    </Button>
  );

  const head = {
    title: copy.title,
    description: atCap ? copy.capReached(RULE_CAPS.sites) : copy.description,
    action: addButton,
  };

  return (
    <>
      {sites.length === 0 ? (
        <Section {...head}>
          <EmptyState title={copy.emptyTitle} description={copy.emptyDescription} />
        </Section>
      ) : (
        <Section {...head} as="list">
          {sites.map((domain) => (
            <SectionRow key={domain} label={domain}>
              {readOnly ? null : (
                <IconButton
                  label={copy.remove(domain)}
                  icon={<X size={ICON_SIZE} aria-hidden="true" />}
                  onClick={() => {
                    edit((current) => removeSite(current, domain));
                  }}
                />
              )}
            </SectionRow>
          ))}
        </Section>
      )}
      <AddSiteSheet
        open={adding}
        sites={sites}
        onAdd={(domain) => {
          edit((current) => addSite(current, domain));
        }}
        onClose={() => {
          setAdding(false);
        }}
      />
    </>
  );
}
