import { test, expect } from '@playwright/test';

test('settings window has passthrough hotkey section', async ({ page }) => {
  await page.goto('/#view=settings');
  await page.waitForLoadState('networkidle');
  await expect(page.getByText('穿透模式快捷鍵')).toBeVisible();
});
