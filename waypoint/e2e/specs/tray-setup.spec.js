// 驗證啟動時 tray 行為：要嘛 setup_tray ok，要嘛失敗時自動 fallback 開列表視窗。
// Steam Deck 環境（無 StatusNotifier）必須走 fallback，使用者才有可視入口。
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";

function logCandidates() {
  const home = os.homedir();
  const xdg = process.env.XDG_STATE_HOME;
  const list = [];
  if (xdg) list.push(path.join(xdg, "waypoint", "error.log"));
  if (home) list.push(path.join(home, ".local/state/waypoint/error.log"));
  if (home) list.push(path.join(home, "waypoint", "error.log"));
  return list;
}

function readLog() {
  for (const p of logCandidates()) {
    try {
      return fs.readFileSync(p, "utf8");
    } catch {
      /* keep trying */
    }
  }
  return null;
}

describe("tray setup", () => {
  it("log 顯示 setup_tray 結果（成功或 fallback 開列表）", async () => {
    // 等 webview 起來代表 setup() 已跑完
    await browser.waitUntil(
      async () => (await browser.getPageSource()).includes("WAYPOINT"),
      { timeout: 20_000, timeoutMsg: "列表視窗未出現，setup() 可能 panic" },
    );
    // 給 write_log_line 一點時間 flush
    await new Promise((r) => setTimeout(r, 500));

    const log = readLog();
    assert.ok(log, `找不到 waypoint log。檢查路徑：${logCandidates().join(", ")}`);

    const ok = log.includes("setup_tray ok");
    const failed = log.includes("setup_tray failed");
    assert.ok(ok || failed, "log 應記錄 setup_tray 成功或失敗，實際內容無相關訊息");

    if (failed) {
      assert.ok(
        log.includes("tray fallback: opening list window"),
        "tray 失敗時必須走 fallback 開列表視窗（避免 Steam Deck 完全無入口）",
      );
    }
  });
});
