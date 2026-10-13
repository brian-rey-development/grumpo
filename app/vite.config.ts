import { resolve } from 'node:path';

import react from '@vitejs/plugin-react';
import { defineConfig } from 'vitest/config';

const DEV_PORT = 1420;
const CHUNK_WARNING_LIMIT_KB = 700;

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  envPrefix: ['VITE_', 'TAURI_ENV_'],
  resolve: { tsconfigPaths: true },
  server: {
    port: DEV_PORT,
    strictPort: true,
    watch: { ignored: ['**/src-tauri/**'] },
  },
  build: {
    target: 'safari16',
    // The bundle is read from local disk inside the app, so the web-network size heuristic does not apply.
    chunkSizeWarningLimit: CHUNK_WARNING_LIMIT_KB,
    rolldownOptions: {
      input: {
        main: resolve(import.meta.dirname, 'index.html'),
        blocker: resolve(import.meta.dirname, 'blocker.html'),
      },
    },
  },
  test: {
    environment: 'jsdom',
    include: ['src/**/*.test.{ts,tsx}'],
    setupFiles: ['src/test/setup.ts'],
  },
});
