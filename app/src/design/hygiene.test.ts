/// <reference types="node" />
import { readdirSync, readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';

import { describe, expect, it } from 'vitest';

import { TOKENS_CSS } from '@/test/tokens';

const SRC_ROOT = resolve(process.cwd(), 'src');
const COMMENT = /\/\*[\s\S]*?\*\//g;
const LITERAL_LENGTH_OR_TIME = /(?<![\w-])-?\d*\.?\d+(?:px|ms|s)\b/g;
const LITERAL_OPACITY = /opacity:\s*0?\.\d+/g;
const LITERAL_BRIGHTNESS = /brightness\(\s*[\d.]/g;
const TOKEN_DECLARATION = /^\s*(--[\w-]+)\s*:/gm;
const SERIES_SLOT_TOKEN = /^--series-\d+$/;

function filesUnder(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    return entry.isDirectory() ? filesUnder(path) : [path];
  });
}

const allFiles = filesUnder(SRC_ROOT).filter((path) => !path.includes('/ipc/generated/'));
const moduleStyles = allFiles.filter((path) => path.endsWith('.module.css'));

function read(path: string): string {
  return readFileSync(path, 'utf8').replace(COMMENT, '');
}

describe('CSS modules', () => {
  it('have at least one stylesheet to check', () => {
    expect(moduleStyles.length).toBeGreaterThan(10);
  });

  it.each(moduleStyles.map((path) => [path.replace(`${SRC_ROOT}/`, ''), path]))(
    '%s takes sizes, radii, opacities and durations from tokens',
    (_name, path) => {
      const css = read(path);
      const offenders = [
        ...css.matchAll(LITERAL_LENGTH_OR_TIME),
        ...css.matchAll(LITERAL_OPACITY),
        ...css.matchAll(LITERAL_BRIGHTNESS),
      ].map((match) => match[0]);
      expect(offenders).toEqual([]);
    },
  );
});

describe('tokens.css', () => {
  it('declares no token that nothing uses', () => {
    const declared = [...TOKENS_CSS.matchAll(TOKEN_DECLARATION)]
      .map((match) => match[1])
      .filter((name): name is string => name !== undefined && !SERIES_SLOT_TOKEN.test(name));
    const sources = allFiles
      .filter((path) => /\.(css|ts|tsx)$/.test(path))
      .filter((path) => !path.endsWith('.test.ts') && !path.endsWith('.test.tsx'))
      .map(read)
      .join('\n');
    const unused = declared.filter((name) => !sources.includes(`var(${name})`));

    expect([...new Set(unused)]).toEqual([]);
  });
});
