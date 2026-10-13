import type { APIRoute } from 'astro';

import { absolute } from '@/config/site';

const AI_CRAWLERS = [
  'GPTBot',
  'OAI-SearchBot',
  'ChatGPT-User',
  'ClaudeBot',
  'Claude-SearchBot',
  'PerplexityBot',
  'Google-Extended',
  'Applebot-Extended',
] as const;

export const GET: APIRoute = ({ site }) => {
  const groups = ['*', ...AI_CRAWLERS].map((agent) => `User-agent: ${agent}\nAllow: /`);
  const body = [...groups, `Sitemap: ${absolute('sitemap.xml', site)}`].join('\n\n');
  return new Response(`${body}\n`, { headers: { 'Content-Type': 'text/plain; charset=utf-8' } });
};
