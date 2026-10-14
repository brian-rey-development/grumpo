import { isBarePublicSuffix } from '@/features/rules/domain';

const COMMON_ALIASES: readonly string[] = ['www.', 'm.'];
const SITE_ALIASES: Readonly<Record<string, readonly string[]>> = {
  'facebook.com': ['mobile.'],
  'reddit.com': ['old.', 'new.', 'i.'],
  'twitter.com': ['mobile.'],
  'x.com': ['mobile.'],
};

/** True for a site one label above a public suffix, such as `youtube.com` or `espn.com.ar`. */
function isRegistrable(site: string): boolean {
  const dot = site.indexOf('.');
  if (dot === -1) return false;
  const parent = site.slice(dot + 1);
  return !parent.includes('.') || isBarePublicSuffix(parent);
}

/**
 * The hostnames the service writes to the hosts file for `site`. It mirrors `hosts_of` in
 * `crates/focus-core/src/hosts.rs`, and both are tested against `tests/vectors/hosts.json`.
 */
export function hostsOf(site: string): string[] {
  if (!isRegistrable(site)) return [site];
  const prefixes = [...COMMON_ALIASES, ...(SITE_ALIASES[site] ?? [])];
  return [site, ...prefixes.map((prefix) => `${prefix}${site}`)];
}
