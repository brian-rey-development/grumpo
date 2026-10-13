import { collapseWhitespace, trimLeadingWhitespace, WHITESPACE_RUNS } from '@/lib/whitespace';

/** The same rule the service applies, so the button never enables for a sentence it would refuse. */
export function phraseMatches(typed: string, sentence: string): boolean {
  return collapseWhitespace(typed) === collapseWhitespace(sentence);
}

interface PhraseProgress {
  /** How many leading characters of the sentence were typed correctly. */
  matched: number;
  /** True when the text typed after those characters is not part of the sentence. */
  wrong: boolean;
}

export function phraseProgress(typed: string, sentence: string): PhraseProgress {
  const text = trimLeadingWhitespace(typed).replace(WHITESPACE_RUNS, ' ');
  let matched = 0;
  while (
    matched < text.length &&
    matched < sentence.length &&
    text[matched] === sentence[matched]
  ) {
    matched += 1;
  }
  return { matched, wrong: collapseWhitespace(text.slice(matched)).length > 0 };
}
