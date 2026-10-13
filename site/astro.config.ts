import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'astro/config';

export default defineConfig({
  site: process.env.SITE_URL ?? 'https://brian-rey-development.github.io',
  base: process.env.SITE_BASE ?? '/',
  trailingSlash: 'ignore',
  devToolbar: { enabled: false },
  build: { assets: 'assets' },
  vite: { plugins: [tailwindcss()] },
});
