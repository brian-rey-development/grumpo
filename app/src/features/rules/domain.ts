import { siteProblemCopy } from '@/copy/rulesFaults';
import { MAX_DOMAIN_LEN, MAX_LABEL_LEN } from '@/ipc/generated/constants';
import type { SiteProblem } from '@/ipc/generated/SiteProblem';
import { trimWhitespace } from '@/lib/whitespace';

const SCHEME_SEPARATOR = '://';
const WWW_PREFIX = 'www.';
const MIN_LABELS = 2;
const SECOND_LEVEL_LABELS: readonly string[] = ['co', 'com', 'net', 'org', 'gov', 'edu', 'ac'];
const HOST_SUFFIX_TERMINATORS = /[/?#]/;
const PORT_SUFFIX = /:\d*$/;
const NON_ASCII = /[\u0080-\uffff]/;
const LABEL_CHARACTERS = /^[a-z0-9-]+$/;
const COUNTRY_CODE = /^[a-z]{2}$/;
const DIGITS_ONLY = /^\d+$/;

type DomainProblem = SiteProblem;

type DomainResult =
  { ok: true; domain: string } | { ok: false; problem: DomainProblem; message: string };

function fail(problem: DomainProblem): DomainResult {
  return { ok: false, problem, message: siteProblemCopy[problem] };
}

function extractHost(input: string): string {
  const separator = input.indexOf(SCHEME_SEPARATOR);
  const rest = separator === -1 ? input : input.slice(separator + SCHEME_SEPARATOR.length);
  const authority = rest.split(HOST_SUFFIX_TERMINATORS)[0] ?? '';
  const hostAndPort = authority.slice(authority.lastIndexOf('@') + 1);
  return hostAndPort.replace(PORT_SUFFIX, '');
}

/** Suffixes like `co.uk` or `com.ar` that no one owns, so blocking one would cut off every site under it. */
export function isBarePublicSuffix(host: string): boolean {
  const dot = host.indexOf('.');
  if (dot === -1) return false;
  const secondLevel = host.slice(0, dot);
  const country = host.slice(dot + 1);
  return SECOND_LEVEL_LABELS.includes(secondLevel) && COUNTRY_CODE.test(country);
}

function stripWww(host: string): string {
  if (!host.startsWith(WWW_PREFIX)) return host;
  const rest = host.slice(WWW_PREFIX.length);
  return rest.includes('.') && !isBarePublicSuffix(rest) ? rest : host;
}

function labelProblem(label: string): DomainProblem | null {
  if (label.length === 0) return 'emptyLabel';
  if (label.length > MAX_LABEL_LEN) return 'labelTooLong';
  if (!LABEL_CHARACTERS.test(label)) return 'invalidCharacter';
  return label.startsWith('-') || label.endsWith('-') ? 'hyphenAtEdge' : null;
}

function hostProblem(host: string): DomainProblem | null {
  if (host.length === 0) return 'empty';
  if (host.length > MAX_DOMAIN_LEN) return 'tooLong';
  const labels = host.split('.');
  const broken = labels.map(labelProblem).find((problem) => problem !== null);
  if (broken !== undefined) return broken;
  if (labels.length < MIN_LABELS) return 'tooFewLabels';
  if (DIGITS_ONLY.test(labels.at(-1) ?? '')) return 'ipAddress';
  return isBarePublicSuffix(host) ? 'publicSuffix' : null;
}

/**
 * Reduces what someone typed or pasted to a bare lowercase host, or says what is wrong with it. It
 * mirrors `Domain::parse` in `crates/focus-core/src/domain.rs`, so the service never refuses what
 * this accepts.
 */
export function normalizeDomainInput(raw: string): DomainResult {
  const trimmed = trimWhitespace(raw);
  if (trimmed.length === 0) return fail('empty');
  if (NON_ASCII.test(trimmed)) return fail('nonAscii');
  const host = extractHost(trimmed.toLowerCase());
  const domain = stripWww(host.endsWith('.') ? host.slice(0, -1) : host);
  const problem = hostProblem(domain);
  return problem === null ? { ok: true, domain } : fail(problem);
}
