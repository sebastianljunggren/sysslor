import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';
import adapter from '@sveltejs/adapter-static';
import { svelteTesting } from '@testing-library/svelte/vite';
import { paraglideVitePlugin } from '@inlang/paraglide-js';

export default defineConfig({
  plugins: [
    sveltekit({
      // An SPA: the Rust binary embeds dist/ and serves index.html for every client route.
      adapter: adapter({ pages: 'dist', assets: 'dist', fallback: 'index.html' }),
      // Family devices keep the tab open for days. Once a new build is deployed, the next
      // navigation is a full page load, so old JS doesn't keep talking to a newer API.
      version: { pollInterval: 10 * 60 * 1000 },
    }),
    paraglideVitePlugin({
      project: './project.inlang',
      outdir: './src/lib/paraglide',
      // The locale isn't part of the URL: a locale picked on the device wins, then the browser's.
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
