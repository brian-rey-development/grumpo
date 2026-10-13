/// <reference types="node" />
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

import { siteProblemCopy } from '@/copy/rulesFaults';
import type { SiteProblem } from '@/ipc/generated/SiteProblem';

const VECTORS_DIRECTORY = resolve(process.cwd(), '../crates/focus-core/tests/vectors');
const HEX_RADIX = 16;
const CODE_POINT_PREFIX = 'U+';

function readVectors(file: string): Record<string, unknown> {
  const parsed: unknown = JSON.parse(readFileSync(resolve(VECTORS_DIRECTORY, file), 'utf8'));
  if (typeof parsed !== 'object' || parsed === null) throw new Error(`${file} is not an object`);
  return Object.fromEntries(Object.entries(parsed));
}

function listAt(vectors: Record<string, unknown>, key: string): unknown[] {
  const list = vectors[key];
  if (!Array.isArray(list)) throw new Error(`${key} is not a list`);
  return list;
}

function pairAt(entry: unknown): [string, string] {
  if (!Array.isArray(entry) || typeof entry[0] !== 'string' || typeof entry[1] !== 'string') {
    throw new Error('A vector is not a pair of strings');
  }
  return [entry[0], entry[1]];
}

function isSiteProblem(name: string): name is SiteProblem {
  return Object.hasOwn(siteProblemCopy, name);
}

function problemPair(entry: unknown): [input: string, problem: SiteProblem] {
  const [input, problem] = pairAt(entry);
  if (!isSiteProblem(problem)) throw new Error(`${problem} is not a site problem`);
  return [input, problem];
}

function characterOf(label: unknown): [label: string, character: string] {
  if (typeof label !== 'string' || !label.startsWith(CODE_POINT_PREFIX)) {
    throw new Error('A whitespace vector is not a code point');
  }
  const codePoint = Number.parseInt(label.slice(CODE_POINT_PREFIX.length), HEX_RADIX);
  return [label, String.fromCodePoint(codePoint)];
}

const domain = readVectors('domain.json');
const whitespace = readVectors('whitespace.json');

/** The cases the service and the app both have to agree on, shared with the Rust tests. */
export const ACCEPTED: [input: string, expected: string][] = listAt(domain, 'accepted').map(pairAt);
export const REJECTED: [input: string, problem: SiteProblem][] = listAt(domain, 'rejected').map(
  problemPair,
);

/** Pairs for `it.each`, so a failing case names the code point instead of an invisible character. */
export const RUST_WHITESPACE: [label: string, character: string][] = listAt(
  whitespace,
  'whitespace',
).map(characterOf);

/** Characters that JavaScript or a person may take for a space, but Rust keeps inside a word. */
export const NOT_RUST_WHITESPACE: [label: string, character: string][] = listAt(
  whitespace,
  'notWhitespace',
).map(characterOf);
