const REPO = 'https://github.com/brian-rey-development/grumpo';
const DOWNLOAD = `${REPO}/releases/latest/download/Grumpo.dmg`;

export const site = {
  name: 'Grumpo',
  title: 'Grumpo, a strict website and app blocker for Mac',
  description:
    'Grumpo blocks distracting websites and apps on your Mac, on a schedule or with daily limits. Unlocking takes a wait and a typed sentence. Free and open source.',
  tagline: 'Grumpo says no. You get things done.',
  author: { name: 'Brian Rey', url: 'https://github.com/brian-rey-development' },
  keywords: [
    'website blocker for Mac',
    'app blocker for Mac',
    'focus app',
    'screen time tracker',
    'self control',
    'distraction blocker',
    'open source',
  ],
  minimumMacOS: '13',
  repo: REPO,
  download: DOWNLOAD,
  checksum: `${DOWNLOAD}.sha256`,
  installScriptPath: 'install.sh',
  releases: `${REPO}/releases`,
  docs: {
    privacy: `${REPO}/blob/main/docs/privacy.md`,
    threatModel: `${REPO}/blob/main/docs/threat-model.md`,
    security: `${REPO}/blob/main/SECURITY.md`,
    license: `${REPO}/blob/main/LICENSE-MIT`,
    uninstall: `${REPO}#uninstall`,
  },
} as const;

export const nav = [
  { label: 'How it works', href: '#how' },
  { label: 'Unlocking', href: '#unlock' },
  { label: 'Privacy', href: '#privacy' },
  { label: 'FAQ', href: '#faq' },
] as const;

export function asset(path: string): string {
  return `${import.meta.env.BASE_URL.replace(/\/$/, '')}/${path.replace(/^\//, '')}`;
}

export function absolute(path: string, base: URL | undefined): string {
  if (base === undefined) throw new Error('astro.config.ts needs a site URL');
  return new URL(asset(path), base).toString();
}

export function installCommand(base: URL | undefined): string {
  return `curl -fsSL ${absolute(site.installScriptPath, base)} | bash`;
}
