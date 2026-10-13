/// <reference types="node" />
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

import { compositeOver, type Rgb, type Rgba, parseCssColor } from '@/test/contrast';

export type Scheme = 'light' | 'dark';
export type TokenSet = Readonly<Record<string, string>>;

const DARK_QUERY = '@media (prefers-color-scheme: dark)';
const DECLARATION = /(--[\w-]+)\s*:\s*([^;]+);/g;
const VAR_REFERENCE = /^var\((--[\w-]+)\)$/;

export const TOKENS_CSS = readFileSync(resolve(process.cwd(), 'src/design/tokens.css'), 'utf8');

function declarations(block: string): Record<string, string> {
  const result: Record<string, string> = {};
  for (const match of block.matchAll(DECLARATION)) {
    const [, name, value] = match;
    if (name !== undefined && value !== undefined) result[name] = value.trim();
  }
  return result;
}

function resolveReferences(tokens: Record<string, string>): TokenSet {
  const resolved: Record<string, string> = {};
  for (const [name, value] of Object.entries(tokens)) {
    let current = value;
    for (let depth = 0; depth < 5; depth += 1) {
      const reference = VAR_REFERENCE.exec(current)?.[1];
      const target = reference === undefined ? undefined : tokens[reference];
      if (target === undefined) break;
      current = target;
    }
    resolved[name] = current;
  }
  return resolved;
}

export function tokensFor(scheme: Scheme): TokenSet {
  const [lightBlock = '', darkBlock = ''] = TOKENS_CSS.split(DARK_QUERY);
  const light = declarations(lightBlock);
  return resolveReferences(scheme === 'light' ? light : { ...light, ...declarations(darkBlock) });
}

export function colorOf(tokens: TokenSet, name: string): Rgba {
  const value = tokens[name];
  const parsed = value === undefined ? null : parseCssColor(value);
  if (parsed === null) throw new Error(`Token ${name} is missing or not a color: ${String(value)}`);
  return parsed;
}

export function opaque(tokens: TokenSet, name: string, over: Rgb): Rgb {
  return compositeOver(colorOf(tokens, name), over);
}
