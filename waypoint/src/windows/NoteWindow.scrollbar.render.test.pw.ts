import { test, expect, type Page } from '@playwright/test';

async function mockNote(page: Page) {
  await page.addInitScript(() => {
    (window as any).__TAURI_INTERNALS__ = {
      metadata: { currentWindow: { label: 'note-x' }, currentWebview: { label: 'note-x', windowLabel: 'note-x' } },
      invoke: (cmd: string) => {
        if (cmd === 'read_note') return Promise.resolve({
          id: 'x', contextId: null, title: 'T',
          content: 'a\n'.repeat(200),
          settings: { fontSize: 14, opacity: 1, hotkey: null, windowBounds: null, passthrough: false },
        });
        if (cmd === 'get_transparent_includes_text') return Promise.resolve(true);
        if (cmd === 'plugin:event|listen') return Promise.resolve(0);
        return Promise.resolve(null);
      },
      transformCallback: () => 0, unregisterCallback: () => {}, convertFileSrc: (s: string) => s,
    };
  });
}

test('筆記 editor scrollbar 視覺寬度為 0', async ({ page }) => {
  await mockNote(page);
  await page.setViewportSize({ width: 320, height: 200 });
  await page.goto('http://localhost:4173/#view=note&noteId=x');
  await page.waitForLoadState('networkidle');
  const result = await page.evaluate(() => {
    const el = document.querySelector('.editor-wrap') as HTMLElement;
    return el ? el.offsetWidth - el.clientWidth : -1;
  });
  expect(result).toBeLessThanOrEqual(0);
});

test('筆記內容 scrollHeight > clientHeight（仍可滾）', async ({ page }) => {
  await mockNote(page);
  await page.setViewportSize({ width: 320, height: 200 });
  await page.goto('http://localhost:4173/#view=note&noteId=x');
  await page.waitForLoadState('networkidle');
  const overflowing = await page.evaluate(() => {
    const el = document.querySelector('.editor-wrap') as HTMLElement;
    return el ? el.scrollHeight > el.clientHeight : false;
  });
  expect(overflowing).toBe(true);
});
