import type { APIRoute } from 'astro';

import { asset, site } from '@/config/site';

const ICON_SIZES = [192, 512] as const;

export const GET: APIRoute = () => {
  const manifest = {
    name: site.name,
    short_name: site.name,
    description: site.description,
    start_url: asset(''),
    display: 'browser',
    background_color: '#fff6ea',
    theme_color: '#fff6ea',
    icons: ICON_SIZES.map((size) => ({
      src: asset(`icon-${size}.png`),
      sizes: `${size}x${size}`,
      type: 'image/png',
    })),
  };
  return new Response(JSON.stringify(manifest, null, 2), {
    headers: { 'Content-Type': 'application/manifest+json' },
  });
};
