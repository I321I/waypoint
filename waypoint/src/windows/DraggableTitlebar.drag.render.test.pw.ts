/**
 * Task 14：四視窗 titlebar mousedown -> cmd_start_dragging 整合測試
 *
 * 透過 mock Tauri invoke 攔截 cmd_start_dragging 呼叫，
 * 對 .draggable-titlebar 模擬 mousedown，驗證帶入正確 label。
 */
import { test, expect, type Page } from "@playwright/test";

async function mockInvokeTracker(page: Page, label: string, withNote = false) {
  await page.addInitScript(([lbl, addNote]: [string, boolean]) => {
    (window as any).__startDraggingCalls = [];
    (window as any).__TAURI_INTERNALS__ = {
      metadata: {
        currentWindow: { label: lbl },
        currentWebview: { label: lbl, windowLabel: lbl },
      },
      invoke: (cmd: string, args: any) => {
        if (cmd === "cmd_start_dragging") {
          (window as any).__startDraggingCalls.push(args?.label ?? null);
          return Promise.resolve(null);
        }
        if (addNote && cmd === "read_note") {
          return Promise.resolve({
            id: "test-id",
            contextId: null,
            title: "Test",
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
  }, [label, withNote] as [string, boolean]);
}

async function clickTitlebarAndAssertLabel(page: Page, expectedLabel: string) {
  const tb = page.locator(".draggable-titlebar").first();
  await expect(tb).toBeVisible({ timeout: 5000 });
  // 點 titlebar 中央空白處（避開內部 button）
  const box = await tb.boundingBox();
  if (!box) throw new Error("titlebar bbox missing");
  await page.mouse.move(box.x + 4, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.up();
  await page.waitForTimeout(100);
  const calls = await page.evaluate(() => (window as any).__startDraggingCalls);
  expect(calls).toContain(expectedLabel);
}

test("ListWindow titlebar mousedown 觸發 cmd_start_dragging(list)", async ({ page }) => {
  await mockInvokeTracker(page, "list");
  await page.goto("http://localhost:4173/#view=list");
  await page.waitForTimeout(800);
  await clickTitlebarAndAssertLabel(page, "list");
});

test("SettingsWindow titlebar mousedown 觸發 cmd_start_dragging(settings)", async ({ page }) => {
  await mockInvokeTracker(page, "settings");
  await page.goto("http://localhost:4173/#view=settings");
  await page.waitForTimeout(800);
  await clickTitlebarAndAssertLabel(page, "settings");
});

test("HelpWindow titlebar mousedown 觸發 cmd_start_dragging(help)", async ({ page }) => {
  await mockInvokeTracker(page, "help");
  await page.goto("http://localhost:4173/#view=help");
  await page.waitForTimeout(800);
  await clickTitlebarAndAssertLabel(page, "help");
});

test("NoteWindow titlebar mousedown 觸發 cmd_start_dragging(note-test-id)", async ({ page }) => {
  await mockInvokeTracker(page, "note-test-id", true);
  await page.goto("http://localhost:4173/#view=note&noteId=test-id");
  await page.waitForTimeout(2000);
  await clickTitlebarAndAssertLabel(page, "note-test-id");
});
