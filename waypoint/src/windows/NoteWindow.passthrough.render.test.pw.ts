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
