import { phraseProgress } from '@/features/unlock/phrase';

import styles from './UnlockSheet.module.css';

interface SentenceDisplayProps {
  sentence: string;
  typed: string;
}

/** Shows the sentence with the correctly typed start dimmed, and the first wrong character marked. */
export function SentenceDisplay({ sentence, typed }: SentenceDisplayProps) {
  const { matched, wrong } = phraseProgress(typed, sentence);
  const done = sentence.slice(0, matched);
  const next = sentence.charAt(matched);
  const rest = sentence.slice(matched + (wrong ? 1 : 0));

  return (
    <blockquote className={styles.sentence}>
      <span className={styles.done}>{done}</span>
      {wrong ? <span className={styles.wrong}>{next}</span> : null}
      <span>{rest}</span>
    </blockquote>
  );
}
