import { type SyntheticEvent, useId, useState } from 'react';

import { Button } from '@/components/Button/Button';
import { Dialog } from '@/components/Dialog/Dialog';
import { SegmentedControl } from '@/components/SegmentedControl/SegmentedControl';
import { TextField } from '@/components/TextField/TextField';
import { commonCopy } from '@/copy/common';
import { ChosenApp } from '@/features/rules/ChosenApp';
import { rulesCopy } from '@/features/rules/copy';
import { normalizeDomainInput } from '@/features/rules/domain';
import type { EditorProps } from '@/features/rules/editorProps';
import { LimitStepper } from '@/features/rules/LimitStepper';
import { addLimit, DEFAULT_LIMIT_MINUTES, hasLimitFor } from '@/features/rules/rulesModel';
import type { AppRef } from '@/ipc/generated/AppRef';
import type { LimitTarget } from '@/ipc/generated/LimitTarget';
import type { Rules } from '@/ipc/generated/Rules';
import { targetName } from '@/lib/limitTarget';

import styles from './AddLimitSheet.module.css';

const copy = rulesCopy.limits.sheet;

type Kind = 'site' | 'app';

const KIND_OPTIONS = [
  { value: 'site', label: copy.site },
  { value: 'app', label: copy.app },
] as const;

interface AddLimitSheetProps extends Pick<EditorProps, 'rules' | 'edit'> {
  open: boolean;
  onClose: () => void;
}

interface FormProps extends Pick<EditorProps, 'rules' | 'edit'> {
  formId: string;
  onDone: () => void;
}

function limitedApps(rules: Rules): ReadonlySet<string> {
  return new Set(
    rules.limits.flatMap(({ target }) => (target.kind === 'app' ? [target.app.bundleId] : [])),
  );
}

interface TargetDraft {
  kind: Kind;
  domain: string;
  app: AppRef | null;
}

/** The target the form describes, or the message that says why it does not describe one yet. */
function resolveTarget({ kind, domain, app }: TargetDraft): LimitTarget | string {
  if (kind === 'app') return app === null ? copy.noApp : { kind: 'app', app };
  const result = normalizeDomainInput(domain);
  return result.ok ? { kind: 'site', domain: result.domain } : result.message;
}

interface TargetFieldsProps {
  draft: TargetDraft;
  rules: Rules;
  error: string | undefined;
  onChange: (next: Partial<TargetDraft>) => void;
}

/** The part of the form that names what to limit, which is a site to type or an app to pick. */
function TargetFields({ draft, rules, error, onChange }: TargetFieldsProps) {
  return (
    <>
      <SegmentedControl
        label={copy.kindLabel}
        options={KIND_OPTIONS}
        value={draft.kind}
        onChange={(kind) => {
          onChange({ kind });
        }}
      />
      {draft.kind === 'site' ? (
        <TextField
          label={copy.siteLabel}
          placeholder={rulesCopy.sites.placeholder}
          value={draft.domain}
          error={error}
          autoCapitalize="off"
          autoCorrect="off"
          autoComplete="off"
          spellCheck={false}
          onChange={(event) => {
            onChange({ domain: event.target.value });
          }}
        />
      ) : (
        <ChosenApp
          app={draft.app}
          excluded={limitedApps(rules)}
          error={error}
          onPick={(app) => {
            onChange({ app });
          }}
          onClear={() => {
            onChange({ app: null });
          }}
        />
      )}
    </>
  );
}

function AddLimitForm({ rules, edit, formId, onDone }: FormProps) {
  const [draft, setDraft] = useState<TargetDraft>({ kind: 'site', domain: '', app: null });
  const [minutes, setMinutes] = useState(DEFAULT_LIMIT_MINUTES);
  const [error, setError] = useState<string | undefined>(undefined);
  const change = (next: Partial<TargetDraft>) => {
    setDraft((current) => ({ ...current, ...next }));
    setError(undefined);
  };

  function submit(event: SyntheticEvent): void {
    event.preventDefault();
    const target = resolveTarget(draft);
    if (typeof target === 'string') {
      setError(target);
      return;
    }
    if (hasLimitFor(rules, target)) {
      setError(copy.duplicate(targetName(target)));
      return;
    }
    edit((current) => addLimit(current, target, minutes));
    onDone();
  }

  return (
    <form id={formId} className={styles.form} onSubmit={submit} noValidate>
      <TargetFields draft={draft} rules={rules} error={error} onChange={change} />
      <div className={styles.minutes}>
        <span>{copy.minutesLabel}</span>
        <LimitStepper
          label={copy.minutesLabel}
          decreaseLabel={copy.decrease}
          increaseLabel={copy.increase}
          value={minutes}
          onChange={setMinutes}
        />
      </div>
    </form>
  );
}

export function AddLimitSheet({ open, rules, edit, onClose }: AddLimitSheetProps) {
  const formId = useId();

  return (
    <Dialog
      open={open}
      title={copy.title}
      description={copy.description}
      onClose={onClose}
      footer={
        <>
          <Button onClick={onClose}>{commonCopy.cancel}</Button>
          <Button type="submit" form={formId} variant="primary">
            {copy.submit}
          </Button>
        </>
      }
    >
      <AddLimitForm rules={rules} edit={edit} formId={formId} onDone={onClose} />
    </Dialog>
  );
}
