import { describe, expect, it } from 'vitest';

import { normalizeDomainInput } from '@/features/rules/domain';
import { siteProblemCopy } from '@/copy/rulesFaults';
import { MAX_DOMAIN_LEN, MAX_LABEL_LEN } from '@/ipc/generated/constants';
import { ACCEPTED, REJECTED } from '@/test/vectors';

const FILLER_LABEL = 'a'.repeat(MAX_LABEL_LEN);
const LONGEST_LABEL_DOMAIN = `${FILLER_LABEL}.com`;
const LAST_LABEL_LENGTH = MAX_DOMAIN_LEN - 3 * (MAX_LABEL_LEN + 1) - '.com'.length;

/** As long as the service accepts, made of three full labels and a shorter one. */
const LONGEST_DOMAIN = `${FILLER_LABEL}.${FILLER_LABEL}.${FILLER_LABEL}.${'b'.repeat(LAST_LABEL_LENGTH)}.com`;

describe('normalizeDomainInput', () => {
  it.each(ACCEPTED)('reduces %j to %s like the service', (input, expected) => {
    expect(normalizeDomainInput(input)).toEqual({ ok: true, domain: expected });
  });

  it.each(REJECTED)('rejects %j as %s like the service', (input, problem) => {
    expect(normalizeDomainInput(input)).toEqual({
      ok: false,
      problem,
      message: siteProblemCopy[problem],
    });
  });

  it('accepts the longest name and label the service accepts', () => {
    expect(LONGEST_DOMAIN).toHaveLength(MAX_DOMAIN_LEN);
    expect(normalizeDomainInput(LONGEST_DOMAIN)).toEqual({ ok: true, domain: LONGEST_DOMAIN });
    expect(normalizeDomainInput(LONGEST_LABEL_DOMAIN).ok).toBe(true);
  });

  it('rejects a label of 64 characters and a name of 250', () => {
    expect(normalizeDomainInput(`a${LONGEST_LABEL_DOMAIN}`)).toMatchObject({
      ok: false,
      problem: 'labelTooLong',
    });
    expect(normalizeDomainInput(`c${LONGEST_DOMAIN}`)).toMatchObject({
      ok: false,
      problem: 'tooLong',
    });
  });

  it('keeps room for the www alias of the longest name', () => {
    expect(normalizeDomainInput(`www.${LONGEST_DOMAIN}`)).toEqual({
      ok: true,
      domain: LONGEST_DOMAIN,
    });
  });

  it('does not strip www when what is left would be a shared suffix', () => {
    expect(normalizeDomainInput('www.co.uk')).toEqual({ ok: true, domain: 'www.co.uk' });
    expect(normalizeDomainInput('www.example.co.uk')).toEqual({
      ok: true,
      domain: 'example.co.uk',
    });
  });

  it('trims the same spaces as the service', () => {
    expect(normalizeDomainInput('\u0085 youtube.com  ')).toEqual({
      ok: true,
      domain: 'youtube.com',
    });
    expect(normalizeDomainInput('﻿youtube.com')).toMatchObject({
      ok: false,
      problem: 'nonAscii',
    });
  });

  it('checks every part of the name before counting them, like the service', () => {
    expect(normalizeDomainInput('exa_mple')).toMatchObject({ problem: 'invalidCharacter' });
    expect(normalizeDomainInput('localhost')).toMatchObject({ problem: 'tooFewLabels' });
  });

  it('puts a specific message on every problem', () => {
    const messages = Object.values(siteProblemCopy);
    expect(new Set(messages).size).toBe(messages.length);
  });
});
