/**
 * 渲染測試 (Rendering Tests) — Playwright
 *
 * 模擬各視窗渲染及所有按鈕的存在性與可點擊性。
 * Tauri API 呼叫（invoke）在瀏覽器模擬中會靜默失敗（.catch()），
 * 視窗結構、按鈕、靜態內容應正確渲染。
 *
 * 執行方式：npm run build && npm run test:render
 */

import { test, expect, type Page } from "@playwright/test";

// 模擬 Tauri API stub（invoke 靜默失敗）
async function mockTauriApis(page: Page, windowLabel = "list") {
  await page.addInitScript((label: string) => {
    (window as any).__TAURI_INTERNALS__ = {
      metadata: {
        currentWindow: { label },
        currentWebview: { label, windowLabel: label },
      },
      invoke: () => Promise.resolve(null),
      transformCallback: () => 0,
      unregisterCallback: () => {},
      convertFileSrc: (s: string) => s,
    };
  }, windowLabel);
}

// mock invoke 能回傳假筆記
async function mockTauriWithNote(page: Page) {
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
}

// ─────────────────────── 白屏防護 ───────────────────────

test("body 背景不是白色（JS 載入前防白屏）", async ({ page }) => {
  await mockTauriApis(page);
  await page.goto("http://localhost:4173/#view=help");

  const bgColor = await page.evaluate(() =>
    window.getComputedStyle(document.body).backgroundColor
  );
  expect(bgColor).not.toBe("rgb(255, 255, 255)");
  expect(bgColor).not.toBe("rgba(0, 0, 0, 0)");
});

// ─────────────────────── 使用說明視窗 ───────────────────────

test.describe("使用說明視窗", () => {
  test("顯示說明標題與內容", async ({ page }) => {
    await mockTauriApis(page, "help");
    await page.goto("http://localhost:4173/#view=help");
    await expect(page.locator("text=Waypoint — 使用說明")).toBeVisible({ timeout: 5000 });
    await expect(page.locator("text=快捷鍵邏輯")).toBeVisible({ timeout: 5000 });
    await expect(page.locator("h2", { hasText: "全域筆記 vs 區域筆記" })).toBeVisible({ timeout: 5000 });
  });

  test("✕ 關閉按鈕存在且可點擊", async ({ page }) => {
    await mockTauriApis(page, "help");
    await page.goto("http://localhost:4173/#view=help");
    await page.waitForTimeout(800);

    const closeBtn = page.locator(".titlebar button").filter({ hasText: "✕" });
    await expect(closeBtn).toBeVisible({ timeout: 5000 });
    // 點擊不應拋出錯誤（invoke 靜默失敗）
    await closeBtn.click();
  });
});

// ─────────────────────── 設定視窗 ───────────────────────

test.describe("設定視窗", () => {
  test("顯示設定標題與內容", async ({ page }) => {
    await mockTauriApis(page, "settings");
    await page.goto("http://localhost:4173/#view=settings");
    await expect(page.locator("text=Waypoint — 設定")).toBeVisible({ timeout: 5000 });
    await expect(page.locator("text=全域快捷鍵")).toBeVisible({ timeout: 5000 });
  });

  test("✕ 關閉按鈕存在且可點擊", async ({ page }) => {
    await mockTauriApis(page, "settings");
    await page.goto("http://localhost:4173/#view=settings");
    await page.waitForTimeout(800);

    const closeBtn = page.locator("button.close-btn");
    await expect(closeBtn).toBeVisible({ timeout: 5000 });
    await closeBtn.click();
  });

  test("套用按鈕存在（快捷鍵設定）", async ({ page }) => {
    await mockTauriApis(page, "settings");
    await page.goto("http://localhost:4173/#view=settings");
    await page.waitForTimeout(800);

    await expect(page.locator("button", { hasText: "套用" })).toBeVisible({ timeout: 5000 });
  });
});

// ─────────────────────── 列表視窗 ───────────────────────

test.describe("列表視窗", () => {
  test("顯示 WAYPOINT 標題", async ({ page }) => {
    await mockTauriApis(page, "list");
    await page.goto("http://localhost:4173/#view=list");
    await expect(page.locator("text=WAYPOINT")).toBeVisible({ timeout: 5000 });
  });

  test("使用說明按鈕（?）存在且可點擊", async ({ page }) => {
    await mockTauriApis(page, "list");
    await page.goto("http://localhost:4173/#view=list");
    await page.waitForTimeout(800);

    const helpBtn = page.locator("button[title='使用說明']");
    await expect(helpBtn).toBeVisible({ timeout: 5000 });
    await helpBtn.click(); // invoke 靜默失敗，不應拋錯
  });

  test("設定按鈕（⚙）存在且可點擊", async ({ page }) => {
    await mockTauriApis(page, "list");
    await page.goto("http://localhost:4173/#view=list");
    await page.waitForTimeout(800);

    const settingsBtn = page.locator("button[title='設定']");
    await expect(settingsBtn).toBeVisible({ timeout: 5000 });
    await settingsBtn.click();
  });

  test("收起全部按鈕（⇊）存在且可點擊", async ({ page }) => {
    await mockTauriApis(page, "list");
    await page.goto("http://localhost:4173/#view=list");
    await page.waitForTimeout(800);

    const collapseBtn = page.locator("button[title='收起全部']");
    await expect(collapseBtn).toBeVisible({ timeout: 5000 });
    await collapseBtn.click();
  });

  test("關閉列表按鈕（✕）存在且可點擊", async ({ page }) => {
    await mockTauriApis(page, "list");
    await page.goto("http://localhost:4173/#view=list");
    await page.waitForTimeout(800);

    const closeBtn = page.locator("button[title='關閉列表']");
    await expect(closeBtn).toBeVisible({ timeout: 5000 });
    await closeBtn.click();
  });
});

// ─────────────────────── 筆記視窗 ───────────────────────

test.describe("筆記視窗", () => {
  test("顯示筆記標題", async ({ page }) => {
    await mockTauriWithNote(page);
    await page.goto("http://localhost:4173/#view=note&noteId=test-id");
    await expect(page.locator("text=Test Note")).toBeVisible({ timeout: 8000 });
  });

  test("最小化按鈕（—）存在且可點擊", async ({ page }) => {
    await mockTauriWithNote(page);
    await page.goto("http://localhost:4173/#view=note&noteId=test-id");
    await page.waitForTimeout(2000);

    const minBtn = page.locator("button[title='最小化']");
    await expect(minBtn).toBeVisible({ timeout: 5000 });
    await minBtn.click();
  });

  test("關閉按鈕（✕）存在且可點擊", async ({ page }) => {
    await mockTauriWithNote(page);
    await page.goto("http://localhost:4173/#view=note&noteId=test-id");
    await page.waitForTimeout(2000);

    const closeBtn = page.locator("button[title='關閉']");
    await expect(closeBtn).toBeVisible({ timeout: 5000 });
    await closeBtn.click();
  });
});
