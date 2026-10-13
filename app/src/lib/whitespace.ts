/**
 * The characters Rust's `char::is_whitespace` accepts, which is what the service splits and trims
 * on. JavaScript's `\s` is not the same set (it lets U+FEFF through and misses U+0085), and text
 * that the interface accepts but the service refuses would be a dead end.
 */
const WHITESPACE_CLASS =
  '[\\t\\n\\v\\f\\r \\u0085\\u00A0\\u1680\\u2000-\\u200A\\u2028\\u2029\\u202F\\u205F\\u3000]';

const WHITESPACE_RUN = new RegExp(`${WHITESPACE_CLASS}+`);
export const WHITESPACE_RUNS = new RegExp(`${WHITESPACE_CLASS}+`, 'g');
const LEADING_WHITESPACE = new RegExp(`^${WHITESPACE_CLASS}+`);
const EDGE_WHITESPACE = new RegExp(`^${WHITESPACE_CLASS}+|${WHITESPACE_CLASS}+$`, 'g');

export function trimLeadingWhitespace(text: string): string {
  return text.replace(LEADING_WHITESPACE, '');
}

export function trimWhitespace(text: string): string {
  return text.replace(EDGE_WHITESPACE, '');
}

/** Trims and joins the words with single spaces, like splitting on whitespace in Rust does. */
export function collapseWhitespace(text: string): string {
  return text
    .split(WHITESPACE_RUN)
    .filter((part) => part.length > 0)
    .join(' ');
}
