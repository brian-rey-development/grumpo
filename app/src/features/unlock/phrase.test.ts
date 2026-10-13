import { describe, expect, it } from 'vitest';

import { phraseMatches, phraseProgress } from '@/features/unlock/phrase';
import { NOT_RUST_WHITESPACE, RUST_WHITESPACE } from '@/test/vectors';
import { UNLOCK_SENTENCE } from '@/test/ipc';

describe('phraseMatches', () => {
  it('accepts the exact sentence', () => {
    expect(phraseMatches(UNLOCK_SENTENCE, UNLOCK_SENTENCE)).toBe(true);
  });

  it('ignores surrounding and repeated whitespace like the service does', () => {
    expect(phraseMatches(`  ${UNLOCK_SENTENCE}  `, UNLOCK_SENTENCE)).toBe(true);
    expect(phraseMatches(UNLOCK_SENTENCE.replace(/ /g, '   '), UNLOCK_SENTENCE)).toBe(true);
    expect(phraseMatches(UNLOCK_SENTENCE.replace(/ /g, '\t'), UNLOCK_SENTENCE)).toBe(true);
  });

  it('does not ignore case, punctuation or missing words', () => {
    expect(phraseMatches(UNLOCK_SENTENCE.toLowerCase(), UNLOCK_SENTENCE)).toBe(false);
    expect(phraseMatches(UNLOCK_SENTENCE.replace(/\.$/, ''), UNLOCK_SENTENCE)).toBe(false);
    expect(phraseMatches('I am choosing', UNLOCK_SENTENCE)).toBe(false);
    expect(phraseMatches('', UNLOCK_SENTENCE)).toBe(false);
  });

  it('does not join words that were typed together', () => {
    expect(phraseMatches('Iam choosing', 'I am choosing')).toBe(false);
  });
});

describe('phraseMatches with unusual spaces', () => {
  it('accepts the sentence typed with any kind of space between the words', () => {
    for (const [, space] of RUST_WHITESPACE) {
      expect(phraseMatches(UNLOCK_SENTENCE.replaceAll(' ', space), UNLOCK_SENTENCE)).toBe(true);
    }
  });

  it.each(NOT_RUST_WHITESPACE)('does not take %s for a space', (_, character) => {
    expect(phraseMatches(`I am${character}choosing`, 'I am choosing')).toBe(false);
  });
});

describe('phraseProgress', () => {
  const sentence = 'I am choosing';

  it('counts the correctly typed start', () => {
    expect(phraseProgress('I am', sentence)).toEqual({ matched: 4, wrong: false });
    expect(phraseProgress('', sentence)).toEqual({ matched: 0, wrong: false });
  });

  it('marks the first wrong character', () => {
    expect(phraseProgress('I amx', sentence)).toEqual({ matched: 4, wrong: true });
    expect(phraseProgress('i am', sentence)).toEqual({ matched: 0, wrong: true });
  });

  it('ignores leading space and collapses repeated spaces', () => {
    expect(phraseProgress('  I   am', sentence)).toEqual({ matched: 4, wrong: false });
  });

  it('treats the same characters as spaces as the service does', () => {
    expect(phraseProgress('I\u00A0am', sentence)).toEqual({ matched: 4, wrong: false });
    expect(phraseProgress('I am choosing\uFEFF', sentence)).toEqual({ matched: 13, wrong: true });
  });

  it('accepts trailing whitespace after the full sentence but not extra text', () => {
    expect(phraseProgress('I am choosing  ', sentence)).toEqual({ matched: 13, wrong: false });
    expect(phraseProgress('I am choosing now', sentence)).toEqual({ matched: 13, wrong: true });
  });
});
