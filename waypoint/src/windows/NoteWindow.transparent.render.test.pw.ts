import { test, expect, type Page } from '@playwright/test';

async function mockNoteWithOpacity(page: Page, opacity: number, transparentIncludesText = true) {
  await page.addInitScript(({ op, tit }) => {
    (window as any).__TAURI_INTERNALS__ = {
      metadata: {
        currentWindow: { label: 'note-test-id' },
        currentWebview: { label: 'note-test-id', windowLabel: 'note-test-id' },
      },
      invoke: (cmd: string) => {
        if (cmd === 'read_note') {
          return Promise.resolve({
            id: 'test-id',
            contextId: null,
            title: 'T',
            content: '',
            settings: { fontSize: 14, opacity: op, hotkey: null, windowBounds: null, passthrough: false },
          });
        }
        if (cmd === 'get_transparent_includes_text') {
          return Promise.resolve(tit);
        }
        if (cmd === 'plugin:event|listen') return Promise.resolve(0);
        return Promise.resolve(null);
      },
      transformCallback: () => 0,
      unregisterCallback: () => {},
      convertFileSrc: (s: string) => s,
    };
  }, { op: opacity, tit: transparentIncludesText });
}

test('note window body 同步加上 note-view class（防白屏）', async ({ page }) => {
  await mockNoteWithOpacity(page, 1);
  await page.goto('http://localhost:4173/#view=note&noteId=test-id');
  await page.waitForLoadState('networkidle');
  const hasClass = await page.evaluate(() => document.body.classList.contains('note-view'));
  expect(hasClass).toBe(true);
});

test('note-window inline style opacity 跟著 settings.opacity', async ({ page }) => {
  await mockNoteWithOpacity(page, 0.3);
  await page.goto('http://localhost:4173/#view=note&noteId=test-id');
  await page.waitForLoadState('networkidle');
  const noteWindow = page.locator('.note-window');
  await expect(noteWindow).toBeVisible({ timeout: 5000 });
  // Wait for config to be applied (transparentIncludesText=true by default)
  await page.waitForTimeout(200);
  const op = await noteWindow.evaluate((el) => getComputedStyle(el).opacity);
  expect(parseFloat(op)).toBeCloseTo(0.3, 2);
});

test('opacity slider 位於 titlebar 內部（在 .draggable-titlebar 之內）', async ({ page }) => {
  await mockNoteWithOpacity(page, 1);
  await page.goto('http://localhost:4173/#view=note&noteId=test-id');
  await page.waitForLoadState('networkidle');
  const sliderInTitlebar = page.locator('.draggable-titlebar .opacity-bar .slider');
  await expect(sliderInTitlebar).toBeVisible({ timeout: 5000 });
});

test('list window body 不會被加上 note-view class', async ({ page }) => {
  await page.addInitScript(() => {
    (window as any).__TAURI_INTERNALS__ = {
      metadata: { currentWindow: { label: 'list' }, currentWebview: { label: 'list', windowLabel: 'list' } },
      invoke: () => Promise.resolve(null),
      transformCallback: () => 0,
      unregisterCallback: () => {},
      convertFileSrc: (s: string) => s,
    };
  });
  await page.goto('http://localhost:4173/#view=list');
  await page.waitForLoadState('networkidle');
  const hasClass = await page.evaluate(() => document.body.classList.contains('note-view'));
  expect(hasClass).toBe(false);
});

test('translucent-text=false 時 .note-window opacity===1，背景含 alpha<1', async ({ page }) => {
  await page.addInitScript(() => {
    (window as any).__TAURI_INTERNALS__ = {
      metadata: {
        currentWindow: { label: 'note-test-id' },
        currentWebview: { label: 'note-test-id', windowLabel: 'note-test-id' },
      },
      invoke: (cmd: string) => {
        if (cmd === 'read_note') return Promise.resolve({
          id: 'test-id', contextId: null, title: 'T', content: '',
          settings: { fontSize: 14, opacity: 0.3, hotkey: null, windowBounds: null, passthrough: false },
        });
        if (cmd === 'get_transparent_includes_text') return Promise.resolve(false);
        if (cmd === 'plugin:event|listen') return Promise.resolve(0);
        return Promise.resolve(null);
      },
      transformCallback: () => 0, unregisterCallback: () => {}, convertFileSrc: (s: string) => s,
    };
  });

  await page.goto('http://localhost:4173/#view=note&noteId=test-id');
  await page.waitForLoadState('networkidle');
  const noteWindow = page.locator('.note-window');
  await noteWindow.waitFor({ state: 'visible' });

  // Wait for config to be applied
  await page.waitForTimeout(200);

  const opVal = await noteWindow.evaluate(el => getComputedStyle(el).opacity);
  expect(parseFloat(opVal)).toBeCloseTo(1, 2);
  const bg = await noteWindow.evaluate(el => getComputedStyle(el).backgroundColor);
  expect(bg).toMatch(/rgba\(/);
  const m = bg.match(/rgba?\([^)]*,\s*([\d.]+)\)/);
  expect(m).not.toBeNull();
  expect(parseFloat(m![1])).toBeLessThan(1);
});
