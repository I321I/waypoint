import { test, expect } from '@playwright/test';

test('passthrough dot exists in titlebar with default green state', async ({ page }) => {
  await page.goto('/#view=note&noteId=test&contextId=null');
  await page.waitForLoadState('networkidle');
  const dot = page.locator('.passthrough-dot');
  await expect(dot).toHaveCount(1);
  await expect(dot).toHaveClass(/dot-on/);
});
