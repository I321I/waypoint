// 驗證 hotkey 真的會被 OS 派送進來並觸發 list 行為。
// 僅在 Linux 跑（用 xdotool 注入按鍵；Windows 路徑由實機驗證，CI 上 SendKeys
// 不可靠且 WM_HOTKEY 已由 Tauri global-shortcut 抽象，註冊成功 log 即視為足夠）。
//
// 流程：列表已自動開啟（WAYPOINT_E2E）→ 按 Ctrl+Shift+Space → 應觸發 CollapseAll
// → log 出現 "hotkey fired: action=CollapseAll"。
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readLog } from "../log-path.js";

function hasXdotool() {
  const r = spawnSync("xdotool", ["--version"], { stdio: "ignore" });
  return r.status === 0;
}

describe("hotkey trigger -> list action (Linux only)", function () {
  before(async function () {
    if (process.platform !== "linux") return this.skip();
    if (!hasXdotool()) return this.skip();
    if (!process.env.DISPLAY) return this.skip();

    await browser.waitUntil(
      async () => (await browser.getPageSource()).includes("WAYPOINT"),
      { timeout: 20_000 },
    );
    await new Promise((r) => setTimeout(r, 500));
  });

  it("按下 Ctrl+Shift+Space 應觸發 hotkey handler（log: hotkey fired）", async () => {
    const before = readLog();
    const beforeFiredCount = (before.match(/hotkey fired/g) || []).length;

    // 注入 OS 級 KeyPress；GTK global-shortcut 應收到並呼叫 callback。
    const r = spawnSync("xdotool", ["key", "ctrl+shift+space"], { stdio: "inherit" });
    assert.equal(r.status, 0, `xdotool 執行失敗：${r.error?.message ?? ""}`);

    // 等 callback + write_log_line flush
    let after = "";
    let afterFiredCount = beforeFiredCount;
    const deadline = Date.now() + 5000;
    while (Date.now() < deadline) {
      after = readLog();
      afterFiredCount = (after.match(/hotkey fired/g) || []).length;
      if (afterFiredCount > beforeFiredCount) break;
      await new Promise((r) => setTimeout(r, 200));
    }

    assert.ok(
      afterFiredCount > beforeFiredCount,
      `hotkey 沒被觸發。before=${beforeFiredCount} after=${afterFiredCount}\nlog 末段：\n${after.slice(-1500)}`,
    );
  });
});
