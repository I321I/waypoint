/**
 * 渲染測試 (Rendering Tests) — Playwright
 *
 * 模擬各種視窗的渲染，驗證 hash routing 正確顯示對應元件。
 * Tauri API 呼叫（invoke 等）在瀏覽器環境下會失敗，但視窗結構
 * 和靜態內容應正確渲染。
 *
 * 執行方式：npm run test:render
 * 需先 npm run build
 */

import { test, expect, type Page } from "@playwright/test";

// 模擬 Tauri API，避免 invoke 呼叫拋出 unhandled error
async function mockTauriApis(page: Page) {
  await page.addInitScript(() => {
    // 提供最小化的 __TAURI_INTERNALS__ stub
    (window as any).__TAURI_INTERNALS__ = {
      metadata: {
        currentWindow: { label: "list" },
        currentWebview: { label: "list", windowLabel: "list" },
      },
      invoke: () => Promise.resolve(null),
      transformCallback: () => 0,
      unregisterCallback: () => {},
      convertFileSrc: (s: string) => s,
    };
  });
}

test.describe("Hash routing 渲染測試", () => {
  test("使用說明視窗：正確顯示說明內容", async ({ page }) => {
    await mockTauriApis(page);
    await page.goto("http://localhost:4173/#view=help");
    await page.waitForTimeout(1000);

    // 確認說明標題出現
    await expect(page.locator("text=Waypoint — 使用說明")).toBeVisible({
      timeout: 5000,
    });
    // 確認說明內容段落出現
    await expect(page.locator("text=快捷鍵邏輯")).toBeVisible({
      timeout: 5000,
    });
  });

  test("設定視窗：正確顯示設定內容", async ({ page }) => {
    await mockTauriApis(page);
    await page.goto("http://localhost:4173/#view=settings");
    await page.waitForTimeout(1000);

    await expect(page.locator("text=Waypoint — 設定")).toBeVisible({
      timeout: 5000,
    });
    await expect(page.locator("text=全域快捷鍵")).toBeVisible({
      timeout: 5000,
    });
  });

  test("列表視窗：正確顯示列表結構", async ({ page }) => {
    await mockTauriApis(page);
    await page.goto("http://localhost:4173/#view=list");
    await page.waitForTimeout(1000);

    // 確認列表視窗的 WAYPOINT 標題出現
    await expect(page.locator("text=WAYPOINT")).toBeVisible({ timeout: 5000 });
  });

  test("筆記視窗：正確顯示（含 placeholder 文字）", async ({ page }) => {
    // mock invoke 回傳假筆記資料
    await page.addInitScript(() => {
      (window as any).__TAURI_INTERNALS__ = {
        metadata: {
          currentWindow: { label: "note-test-id" },
          currentWebview: { label: "note-test-id", windowLabel: "note-test-id" },
        },
        invoke: (cmd: string) => {
          if (cmd === "read_note") {
            return Promise.resolve({
              id: "test-id",
              contextId: null,
              title: "Test Note",
              content: "",
              settings: { fontSize: 14, opacity: 1, hotkey: null, windowBounds: null },
            });
          }
          return Promise.resolve(null);
        },
        transformCallback: () => 0,
        unregisterCallback: () => {},
        convertFileSrc: (s: string) => s,
      };
    });

    await page.goto("http://localhost:4173/#view=note&noteId=test-id");
    await page.waitForTimeout(2000);

    // 確認筆記標題列出現
    await expect(page.locator("text=Test Note")).toBeVisible({ timeout: 8000 });
  });

  test("body 背景不是白色（防止白屏）", async ({ page }) => {
    await mockTauriApis(page);
    await page.goto("http://localhost:4173/#view=help");

    const bgColor = await page.evaluate(() => {
      return window.getComputedStyle(document.body).backgroundColor;
    });

    // body 背景應該是深色 (#1e1e1e = rgb(30,30,30))，不是白色
    expect(bgColor).not.toBe("rgb(255, 255, 255)");
    expect(bgColor).not.toBe("rgba(0, 0, 0, 0)");
  });
});
