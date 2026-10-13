import { Plus } from 'lucide-react';
import { useState } from 'react';

import { Button } from '@/components/Button/Button';
import { EmptyState } from '@/components/EmptyState/EmptyState';
import { Section } from '@/components/Section/Section';
import { AddLimitSheet } from '@/features/rules/AddLimitSheet';
import { rulesCopy } from '@/features/rules/copy';
import type { EditorProps } from '@/features/rules/editorProps';
import { LimitEditRow } from '@/features/rules/LimitEditRow';
import { removeLimit, RULE_CAPS, setLimitMinutes } from '@/features/rules/rulesModel';
import { useLimitUsage } from '@/ipc/queries';
import { targetKey } from '@/lib/limitTarget';

const ICON_SIZE = 14;

export function LimitsEditor({ rules, edit, readOnly }: EditorProps) {
  const copy = rulesCopy.limits;
  const [adding, setAdding] = useState(false);
  const usage = useLimitUsage();
  const usedByTarget = new Map(
    (usage.data ?? []).map((entry) => [targetKey(entry.target), entry.usedSeconds]),
  );
  const atCap = rules.limits.length >= RULE_CAPS.limits;
  const description = atCap ? copy.capReached(RULE_CAPS.limits) : copy.description;
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
    description,
    action: addButton,
  };

  return (
    <>
      {rules.limits.length === 0 ? (
        <Section {...head}>
          <EmptyState title={copy.emptyTitle} description={copy.emptyDescription} />
        </Section>
      ) : (
        <Section {...head} as="list">
          {rules.limits.map((limit) => (
            <LimitEditRow
              key={targetKey(limit.target)}
              limit={limit}
              usedSeconds={usedByTarget.get(targetKey(limit.target))}
              readOnly={readOnly}
              onChange={(minutes) => {
                edit((current) => setLimitMinutes(current, limit.target, minutes));
              }}
              onRemove={() => {
                edit((current) => removeLimit(current, limit.target));
              }}
            />
          ))}
        </Section>
      )}
      <AddLimitSheet
        open={adding}
        rules={rules}
        edit={edit}
        onClose={() => {
          setAdding(false);
        }}
      />
    </>
  );
}
