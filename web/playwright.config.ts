import { defineConfig, devices } from '@playwright/test';
import { DEMO_NOW, PORT } from './e2e/demo';

// Only for the README screenshots. Fonts and rendering differ between machines, so CI runs
// this in the Playwright container whose version matches @playwright/test.
export default defineConfig({
  testDir: 'e2e',
  // Paths relative to the repo root keep the README links simple.
  snapshotPathTemplate: '../docs/screenshots/{arg}{ext}',
  use: {
    ...devices['Desktop Chrome'],
    baseURL: `http://127.0.0.1:${PORT}`,
    viewport: { width: 390, height: 844 },
    deviceScaleFactor: 2,
    isMobile: false,
    locale: 'en-US',
    timezoneId: 'Europe/Stockholm',
  },
  expect: {
    toHaveScreenshot: { animations: 'disabled', caret: 'hide', scale: 'device' },
  },
  webServer: {
    command: './e2e/serve-demo.sh',
    url: `http://127.0.0.1:${PORT}/healthz`,
    env: { PORT: String(PORT), SYSSLOR_DEMO_NOW: DEMO_NOW },
    reuseExistingServer: false,
    stdout: 'pipe',
  },
});
