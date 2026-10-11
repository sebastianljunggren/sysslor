import { expect, test } from '@playwright/test';
import { DEMO_NOW } from './demo';

test.beforeEach(async ({ page }) => {
  await page.clock.setFixedTime(new Date(DEMO_NOW));
  await page.addInitScript(() => localStorage.setItem('PARAGLIDE_LOCALE', 'en'));
  await page.goto('/');
  await page.getByLabel('Password').fill('demo');
  await page.getByRole('button', { name: 'Log in' }).click();
  await expect(page.getByText('Clean range hood filter')).toBeVisible();
  // The container has no proper UI font, so `system-ui` falls back to a Chinese one. Liberation
  // Sans ships with the image and looks like a typical system font.
  await page.addStyleTag({ content: 'body { font-family: "Liberation Sans", sans-serif; }' });
});

for (const colorScheme of ['light', 'dark'] as const) {
  test(`task list, ${colorScheme}`, async ({ page }) => {
    await page.emulateMedia({ colorScheme });
    await expect(page).toHaveScreenshot(`tasks-${colorScheme}.png`);
  });
}
