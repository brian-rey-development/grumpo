import { type SyntheticEvent, useState } from 'react';

import { Button } from '@/components/Button/Button';
import { TextField } from '@/components/TextField/TextField';
import { commonCopy } from '@/copy/common';
import { describeFailure } from '@/copy/failures';
import { unlockCopy } from '@/features/unlock/copy';
import { DurationChoice } from '@/features/unlock/DurationChoice';
import { FailureMessage } from '@/features/unlock/FailureMessage';
import { phraseMatches } from '@/features/unlock/phrase';
import { ReasonNote } from '@/features/unlock/ReasonNote';
import { SentenceDisplay } from '@/features/unlock/SentenceDisplay';
import { useRefreshWhenElapsed, useSecondsUntil } from '@/features/unlock/useCountdown';
import type { PhaseViewProps } from '@/features/unlock/viewProps';
import { UNLOCK_DURATIONS_MINUTES } from '@/ipc/generated/constants';
import type { UnlockView } from '@/ipc/generated/UnlockView';
import { formatCountdown } from '@/lib/format';

import styles from './UnlockSheet.module.css';

/** The middle of the lengths the service offers. */
const PREFERRED_MINUTES = UNLOCK_DURATIONS_MINUTES[1];

function defaultMinutes(durations: readonly number[]): number {
  return durations.includes(PREFERRED_MINUTES)
    ? PREFERRED_MINUTES
    : (durations[0] ?? UNLOCK_DURATIONS_MINUTES[0]);
}

interface ReadyViewProps extends PhaseViewProps {
  unlock: UnlockView;
  expiresAt: number;
}

export function ReadyView({ unlock, expiresAt, actions, reason }: ReadyViewProps) {
  const copy = unlockCopy.ready;
  const [typed, setTyped] = useState('');
  const [minutes, setMinutes] = useState(() => defaultMinutes(unlock.durations));
  const remaining = useSecondsUntil(expiresAt);
  useRefreshWhenElapsed(expiresAt);
  const matches = phraseMatches(typed, unlock.sentence);
  const timeIsUp = remaining === 0;
  const canConfirm = matches && !timeIsUp;
  const phraseFailure = actions.failure?.code === 'phraseMismatch' ? actions.failure : null;
  const otherFailure = phraseFailure === null ? actions.failure : null;

  function submit(event: SyntheticEvent): void {
    event.preventDefault();
    if (canConfirm && !actions.pending) actions.confirm(typed, minutes);
  }

  return (
    <form className={styles.view} onSubmit={submit} noValidate>
      <ReasonNote reason={reason} />
      <p className={styles.instruction}>{copy.instruction}</p>
      <SentenceDisplay sentence={unlock.sentence} typed={typed} />
      <TextField
        label={copy.fieldLabel}
        value={typed}
        preventPaste
        data-autofocus
        data-matches={matches || undefined}
        className={styles.phrase}
        hint={matches ? copy.matched : undefined}
        error={
          phraseFailure === null
            ? undefined
            : describeFailure(phraseFailure, commonCopy.tryAgainSoon)
        }
        onChange={(event) => {
          setTyped(event.target.value);
        }}
      />
      <DurationChoice durations={unlock.durations} minutes={minutes} onChange={setMinutes} />
      <p className={styles.secondary}>
        {timeIsUp ? copy.timeUp : copy.deadline(formatCountdown(remaining, { padMinutes: false }))}
      </p>
      <p className="sr-only" role="status">
        {timeIsUp ? copy.timeUp : ''}
      </p>
      <FailureMessage failure={otherFailure} />
      <div className={styles.actions}>
        <Button type="button" disabled={actions.pending} onClick={actions.cancel}>
          {copy.cancel}
        </Button>
        <Button type="submit" variant="primary" loading={actions.pending} disabled={!canConfirm}>
          {copy.confirm(minutes)}
        </Button>
      </div>
    </form>
  );
}
