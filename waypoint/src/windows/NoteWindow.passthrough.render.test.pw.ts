import { test, expect, type Page } from '@playwright/test';

async function mockTauriWithNote(page: Page) {
  await page.addInitScript(() => {
    (window as any).__TAURI_INTERNALS__ = {
      metadata: {
        currentWindow: { label: "note-test" },
        currentWebview: { label: "note-test", windowLabel: "note-test" },
      },
      invoke: (cmd: string) => {
        if (cmd === "read_note") {
          return Promise.resolve({
            id: "test",
            contextId: null,
            title: "T",
            content: "",
            settings: { fontSize: 14, opacity: 1, hotkey: null, windowBounds: null, passthrough: false },
          });
        }
        if (cmd === "get_transparent_includes_text") return Promise.resolve(true);
        if (cmd === "plugin:event|listen") return Promise.resolve(0);
        return Promise.resolve(null);
      },
      transformCallback: () => 0,
      unregisterCallback: () => {},
      convertFileSrc: (s: string) => s,
    };
  });
}

test('passthrough dot exists in titlebar with default green state', async ({ page }) => {
  await mockTauriWithNote(page);
  await page.goto('/#view=note&noteId=test&contextId=null');
  await page.waitForTimeout(1000);
  const dot = page.locator('.passthrough-dot');
  await expect(dot).toHaveCount(1);
  await expect(dot).toHaveClass(/dot-on/);
});

test('passthrough=true 時 .draggable-titlebar 不存在，title-input 與 editor 仍在', async ({ page }) => {
  await page.addInitScript(() => {
    (window as any).__TAURI_INTERNALS__ = {
      metadata: { currentWindow: { label: 'note-x' }, currentWebview: { label: 'note-x', windowLabel: 'note-x' } },
      invoke: (cmd: string) => {
        if (cmd === 'read_note') return Promise.resolve({
          id: 'x', contextId: null, title: 'T', content: 'hi',
          settings: { fontSize: 14, opacity: 1, hotkey: null, windowBounds: null, passthrough: true },
        });
        if (cmd === 'get_transparent_includes_text') return Promise.resolve(true);
        if (cmd === 'plugin:event|listen') return Promise.resolve(0);
        return Promise.resolve(null);
      },
      transformCallback: () => 0, unregisterCallback: () => {}, convertFileSrc: (s: string) => s,
    };
  });
  await page.goto('/#view=note&noteId=x');
  await page.waitForTimeout(1000);
  await expect(page.locator('.draggable-titlebar')).toHaveCount(0);
  await expect(page.locator('.title-input')).toBeVisible();
  await expect(page.locator('.editor-wrap')).toBeVisible();
});
