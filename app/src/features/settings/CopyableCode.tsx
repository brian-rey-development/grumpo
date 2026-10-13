import { Check, Copy } from 'lucide-react';
import { useEffect, useState } from 'react';

import { Button } from '@/components/Button/Button';
import { settingsCopy } from '@/features/settings/copy';

import styles from './CopyableCode.module.css';

const COPIED_RESET_MS = 2_000;
const ICON_SIZE = 14;
const copy = settingsCopy.service;

type CopyState = 'idle' | 'copied' | 'failed';

interface CopyableCodeProps {
  label: string;
  code: string;
}

export function CopyableCode({ label, code }: CopyableCodeProps) {
  const [state, setState] = useState<CopyState>('idle');

  useEffect(() => {
    if (state !== 'copied') return undefined;
    const timer = window.setTimeout(() => {
      setState('idle');
    }, COPIED_RESET_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, [state]);

  async function copyCode(): Promise<void> {
    try {
      await navigator.clipboard.writeText(code);
      setState('copied');
    } catch {
      setState('failed');
    }
  }

  const Icon = state === 'copied' ? Check : Copy;
  return (
    <div className={styles.block}>
      <div className={styles.row}>
        <code className={styles.code} aria-label={label} data-selectable>
          {code}
        </code>
        <Button
          size="small"
          icon={<Icon size={ICON_SIZE} aria-hidden="true" />}
          onClick={() => void copyCode()}
        >
          {state === 'copied' ? copy.copied : copy.copy}
        </Button>
      </div>
      {state === 'failed' ? (
        <p className={styles.error} role="alert">
          {copy.copyFailed}
        </p>
      ) : null}
    </div>
  );
}
