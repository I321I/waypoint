import { test, expect, type Page } from '@playwright/test';

async function mockNote(page: Page, contextId: string | null, title: string) {
  await page.addInitScript(({ ctx, t }) => {
    (window as any).__TAURI_INTERNALS__ = {
      metadata: { currentWindow: { label: 'note-x' }, currentWebview: { label: 'note-x', windowLabel: 'note-x' } },
      invoke: (cmd: string) => {
        if (cmd === 'read_note') return Promise.resolve({
          id: 'x', contextId: ctx, title: t, content: '',
          settings: { fontSize: 14, opacity: 1, hotkey: null, windowBounds: null, passthrough: false },
        });
        if (cmd === 'get_transparent_includes_text') return Promise.resolve(true);
        if (cmd === 'plugin:event|listen') return Promise.resolve(0);
        return Promise.resolve(null);
      },
      transformCallback: () => 0, unregisterCallback: () => {}, convertFileSrc: (s: string) => s,
    };
  }, { ctx: contextId, t: title });
}

test('全域筆記 titlebar 顯示 {title}-Global', async ({ page }) => {
  await mockNote(page, null, '1122');
  await page.goto('http://localhost:4173/#view=note&noteId=x');
  await page.waitForLoadState('networkidle');
  await expect(page.locator('.note-title')).toHaveText('1122-Global');
});

test('區域筆記 titlebar 顯示 {title}-{contextId}', async ({ page }) => {
  await mockNote(page, 'edge', '我是誰');
  await page.goto('http://localhost:4173/#view=note&noteId=x&contextId=edge');
  await page.waitForLoadState('networkidle');
  await expect(page.locator('.note-title')).toHaveText('我是誰-edge');
});

test('NoteWindow 不再顯示 .statusbar', async ({ page }) => {
  await mockNote(page, null, 'T');
  await page.goto('http://localhost:4173/#view=note&noteId=x');
  await page.waitForLoadState('networkidle');
  await expect(page.locator('.statusbar')).toHaveCount(0);
});
