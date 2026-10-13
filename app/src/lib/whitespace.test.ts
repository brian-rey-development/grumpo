import { describe, expect, it } from 'vitest';

import { collapseWhitespace, trimLeadingWhitespace, trimWhitespace } from '@/lib/whitespace';
import { NOT_RUST_WHITESPACE, RUST_WHITESPACE } from '@/test/vectors';

describe('collapseWhitespace', () => {
  it('trims and joins with single spaces', () => {
    expect(collapseWhitespace('  a \n b\t\tc ')).toBe('a b c');
    expect(collapseWhitespace('   ')).toBe('');
    expect(collapseWhitespace('')).toBe('');
  });

  it.each(RUST_WHITESPACE)('treats %s as a space like the service does', (_, space) => {
    expect(collapseWhitespace(`a${space}${space}b`)).toBe('a b');
    expect(collapseWhitespace(`${space}a${space}`)).toBe('a');
  });

  it.each(NOT_RUST_WHITESPACE)('keeps %s inside the text', (_, character) => {
    expect(collapseWhitespace(`a${character}b`)).toBe(`a${character}b`);
    expect(collapseWhitespace(` ${character} `)).toBe(character);
  });
});

describe('trimWhitespace', () => {
  it.each(RUST_WHITESPACE)('trims %s from both ends only', (_, space) => {
    expect(trimWhitespace(`${space}a${space}b${space}`)).toBe(`a${space}b`);
  });

  it.each(NOT_RUST_WHITESPACE)('leaves %s alone', (_, character) => {
    expect(trimWhitespace(`${character}a${character}`)).toBe(`${character}a${character}`);
  });
});

describe('trimLeadingWhitespace', () => {
  it('keeps the trailing whitespace', () => {
    expect(trimLeadingWhitespace('  a b 　')).toBe('a b 　');
  });
});
