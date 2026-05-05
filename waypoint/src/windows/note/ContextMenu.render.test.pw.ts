import { test, expect, type Page } from '@playwright/test';

async function mockNote(page: Page) {
  await page.addInitScript(() => {
    (window as any).__TAURI_INTERNALS__ = {
      metadata: { currentWindow: { label: 'note-x' }, currentWebview: { label: 'note-x', windowLabel: 'note-x' } },
      invoke: (cmd: string, args: any) => {
        if (cmd === 'read_note') return Promise.resolve({
          id: 'x', contextId: null, title: 'T', content: 'hello',
          settings: { fontSize: 14, opacity: 1, hotkey: null, windowBounds: null, passthrough: false },
        });
        if (cmd === 'get_transparent_includes_text') return Promise.resolve(true);
        if (cmd === 'plugin:event|listen') return Promise.resolve(0);
        if (cmd === 'save_note_settings') {
          (window as any).__lastSavedFontSize = args.settings.fontSize;
          return Promise.resolve(null);
        }
        if (cmd === 'delete_note') {
          (window as any).__deleteCalled = true;
          return Promise.resolve(null);
        }
        return Promise.resolve(null);
      },
      transformCallback: () => 0, unregisterCallback: () => {}, convertFileSrc: (s: string) => s,
    };
  });
}

test('右鍵 editor 區會顯示 ContextMenu，含 4 項', async ({ page }) => {
  await mockNote(page);
  await page.goto('http://localhost:4173/#view=note&noteId=x');
  await page.waitForLoadState('networkidle');
  await page.locator('.editor-area').click({ button: 'right' });
  await expect(page.locator('.context-menu')).toBeVisible();
  await expect(page.locator('.context-menu [data-mi="copy"]')).toBeVisible();
  await expect(page.locator('.context-menu [data-mi="paste"]')).toBeVisible();
  await expect(page.locator('.context-menu [data-mi="font-size"]')).toBeVisible();
  await expect(page.locator('.context-menu [data-mi="delete"]')).toBeVisible();
});

test('字體大小 stepper + 會 dispatch save_note_settings', async ({ page }) => {
  await mockNote(page);
  await page.goto('http://localhost:4173/#view=note&noteId=x');
  await page.waitForLoadState('networkidle');
  await page.locator('.editor-area').click({ button: 'right' });
  await page.locator('.context-menu [data-mi="font-size"] [data-act="inc"]').click();
  await page.waitForTimeout(100);
  expect(await page.evaluate(() => (window as any).__lastSavedFontSize)).toBe(15);
});

test('刪除此筆記會走 ConfirmDialog → delete_note', async ({ page }) => {
  await mockNote(page);
  await page.goto('http://localhost:4173/#view=note&noteId=x');
  await page.waitForLoadState('networkidle');
  await page.locator('.editor-area').click({ button: 'right' });
  await page.locator('.context-menu [data-mi="delete"]').click();
  await expect(page.locator('.dialog')).toBeVisible();
  await page.locator('.dialog button.danger').click();
  await page.waitForTimeout(100);
  expect(await page.evaluate(() => (window as any).__deleteCalled)).toBe(true);
});

test('Escape 關閉 ContextMenu', async ({ page }) => {
  await mockNote(page);
  await page.goto('http://localhost:4173/#view=note&noteId=x');
  await page.waitForLoadState('networkidle');
  await page.locator('.editor-area').click({ button: 'right' });
  await expect(page.locator('.context-menu')).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(page.locator('.context-menu')).toHaveCount(0);
});
