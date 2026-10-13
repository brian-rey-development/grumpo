import { type SyntheticEvent, useId, useState } from 'react';

import { Button } from '@/components/Button/Button';
import { Dialog } from '@/components/Dialog/Dialog';
import { TextField } from '@/components/TextField/TextField';
import { commonCopy } from '@/copy/common';
import { rulesCopy } from '@/features/rules/copy';
import { normalizeDomainInput } from '@/features/rules/domain';

const copy = rulesCopy.sites;

interface AddSiteSheetProps {
  open: boolean;
  sites: readonly string[];
  onAdd: (domain: string) => void;
  onClose: () => void;
}

interface AddSiteFormProps extends Pick<AddSiteSheetProps, 'sites' | 'onAdd'> {
  formId: string;
  onDone: () => void;
}

function AddSiteForm({ formId, sites, onAdd, onDone }: AddSiteFormProps) {
  const [value, setValue] = useState('');
  const [error, setError] = useState<string | undefined>(undefined);

  function submit(event: SyntheticEvent): void {
    event.preventDefault();
    const result = normalizeDomainInput(value);
    if (!result.ok) {
      setError(result.message);
      return;
    }
    if (sites.includes(result.domain)) {
      setError(copy.duplicate(result.domain));
      return;
    }
    onAdd(result.domain);
    onDone();
  }

  return (
    <form id={formId} onSubmit={submit} noValidate>
      <TextField
        label={copy.inputLabel}
        placeholder={copy.placeholder}
        value={value}
        error={error}
        autoCapitalize="off"
        autoCorrect="off"
        autoComplete="off"
        spellCheck={false}
        onChange={(event) => {
          setValue(event.target.value);
          setError(undefined);
        }}
      />
    </form>
  );
}

export function AddSiteSheet({ open, sites, onAdd, onClose }: AddSiteSheetProps) {
  const formId = useId();

  return (
    <Dialog
      open={open}
      title={copy.sheetTitle}
      onClose={onClose}
      footer={
        <>
          <Button onClick={onClose}>{commonCopy.cancel}</Button>
          <Button type="submit" form={formId} variant="primary">
            {copy.add}
          </Button>
        </>
      }
    >
      <AddSiteForm formId={formId} sites={sites} onAdd={onAdd} onDone={onClose} />
    </Dialog>
  );
}
