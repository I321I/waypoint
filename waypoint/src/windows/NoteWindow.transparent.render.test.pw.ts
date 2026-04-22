import { test, expect } from '@playwright/test';

test('note window body has note-view class and rgba background', async ({ page }) => {
  await page.goto('/#view=note&noteId=test&contextId=null');
  await page.waitForLoadState('networkidle');
  // class 應在 onMount 同步加上（不依賴 await notesApi.read）
  const hasClass = await page.evaluate(() => document.body.classList.contains('note-view'));
  expect(hasClass).toBe(true);
  // 設 alpha=0.5 讓瀏覽器以 rgba 形式回報（alpha=1 時瀏覽器會省略為 rgb）
  await page.evaluate(() => document.documentElement.style.setProperty('--note-alpha', '0.5'));
  const bg = await page.evaluate(() => getComputedStyle(document.body).backgroundColor);
  expect(bg).toMatch(/^rgba\(/);
});

test('note window alpha follows --note-alpha CSS variable', async ({ page }) => {
  await page.goto('/#view=note&noteId=test&contextId=null');
  await page.waitForLoadState('networkidle');
  await page.evaluate(() => document.documentElement.style.setProperty('--note-alpha', '0.3'));
  const bg = await page.evaluate(() => getComputedStyle(document.body).backgroundColor);
  // rgba(30, 30, 30, 0.3)
  expect(bg).toMatch(/rgba\(30,\s*30,\s*30,\s*0\.3\)/);
});

test('list window body does not have note-view class', async ({ page }) => {
  await page.goto('/#view=list');
  await page.waitForLoadState('networkidle');
  const hasClass = await page.evaluate(() => document.body.classList.contains('note-view'));
  expect(hasClass).toBe(false);
});
