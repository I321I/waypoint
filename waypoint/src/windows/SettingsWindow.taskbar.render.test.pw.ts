import { test, expect } from '@playwright/test';

test('SettingsWindow contains 工作列圖示 section', async ({ page }) => {
  await page.goto('/#view=settings');
  await page.waitForLoadState('networkidle');
  await expect(page.locator('text=工作列圖示')).toBeVisible();
});
