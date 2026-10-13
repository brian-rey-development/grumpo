import type { APIRoute } from 'astro';

import { absolute } from '@/config/site';

const PAGES = [''] as const;

export const GET: APIRoute = ({ site }) => {
  const lastModified = new Date().toISOString().slice(0, 10);
  const urls = PAGES.map(
    (path) => `  <url><loc>${absolute(path, site)}</loc><lastmod>${lastModified}</lastmod></url>`,
  );
  const body = [
    '<?xml version="1.0" encoding="UTF-8"?>',
    '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">',
    ...urls,
    '</urlset>',
  ].join('\n');
  return new Response(`${body}\n`, {
    headers: { 'Content-Type': 'application/xml; charset=utf-8' },
  });
};
