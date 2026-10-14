import type { APIRoute } from 'astro';

import { site } from '@/config/site';
import template from '@/installer/install.sh?raw';

const PLACEHOLDER = /@([A-Z_]+)@/g;

const substitutions: Readonly<Record<string, string>> = {
  REPO_URL: site.repo,
  DMG_URL: site.download,
  CHECKSUM_URL: site.checksum,
  MIN_MACOS: site.minimumMacOS,
};

function render(source: string): string {
  return source.replace(PLACEHOLDER, (_match, key: string) => {
    const value = substitutions[key];
    if (value === undefined) throw new Error(`install.sh has no value for @${key}@`);
    return value;
  });
}

export const GET: APIRoute = () =>
  new Response(render(template), { headers: { 'Content-Type': 'text/plain; charset=utf-8' } });
