import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { svelteTesting } from '@testing-library/svelte/vite';
import { paraglideVitePlugin } from '@inlang/paraglide-js';

export default defineConfig({
  plugins: [
    svelte(),
    paraglideVitePlugin({
      project: './project.inlang',
      outdir: './src/lib/paraglide',
      // No URL routing in this SPA: a locale picked on the device wins, then the browser's.
      strategy: ['localStorage', 'preferredLanguage', 'baseLocale'],
    }),
    process.env.VITEST && svelteTesting(),
  ],
  server: {
    proxy: {
      '/api': 'http://localhost:8080',
      '/healthz': 'http://localhost:8080',
    },
  },
  test: {
    environment: 'jsdom',
    include: ['src/**/*.test.ts'],
    setupFiles: ['./src/test/setup.ts'],
    // A zone with DST, so day math that assumes 24-hour days breaks in tests too.
    env: { TZ: 'Europe/Stockholm' },
    unstubGlobals: true,
  },
});
