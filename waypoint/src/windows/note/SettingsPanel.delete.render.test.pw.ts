import { test, expect } from '@playwright/test';

test('SettingsPanel 顯示「刪除此筆記」按鈕，點擊後彈 ConfirmDialog', async ({ page }) => {
  await page.addInitScript(() => {
    (window as any).__TAURI_INTERNALS__ = {
      metadata: { currentWindow: { label: 'note-x' }, currentWebview: { label: 'note-x', windowLabel: 'note-x' } },
      invoke: (cmd: string) => {
        if (cmd === 'read_note') return Promise.resolve({
          id: 'x', contextId: null, title: 'T', content: '',
          settings: { fontSize: 14, opacity: 1, hotkey: null, windowBounds: null, passthrough: false },
        });
        if (cmd === 'get_transparent_includes_text') return Promise.resolve(true);
        if (cmd === 'plugin:event|listen') return Promise.resolve(0);
        return Promise.resolve(null);
      },
      transformCallback: () => 0, unregisterCallback: () => {}, convertFileSrc: (s:string)=>s,
    };
  });

  await page.goto('http://localhost:4173/#view=note&noteId=x');
  await page.waitForLoadState('networkidle');

  // Open the settings panel via the toolbar's settings button
  await page.locator('[title="設定"]').click();

  const delBtn = page.locator('button[data-testid="delete-this-note"]');
  await expect(delBtn).toBeVisible();
  await delBtn.click();
  await expect(page.locator('.dialog .msg')).toContainText('刪除');
});
