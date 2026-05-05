// 驗證主 hotkey（預設 Ctrl+Shift+Space）有成功註冊。
// 失敗訊號：log 出現 "register_hotkey failed" 或完全沒記錄這行。
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";

function readLog() {
  const home = os.homedir();
  const xdg = process.env.XDG_STATE_HOME;
  const candidates = [];
  if (xdg) candidates.push(path.join(xdg, "waypoint", "error.log"));
  if (home) candidates.push(path.join(home, ".local/state/waypoint/error.log"));
  if (home) candidates.push(path.join(home, "waypoint", "error.log"));
  for (const p of candidates) {
    try { return fs.readFileSync(p, "utf8"); } catch {}
  }
  return null;
}

describe("hotkey register", () => {
  before(async () => {
    await browser.waitUntil(
      async () => (await browser.getPageSource()).includes("WAYPOINT"),
      { timeout: 20_000 },
    );
    await new Promise((r) => setTimeout(r, 500));
  });

  it("主 hotkey 成功註冊（log: register_hotkey ok: Ctrl+Shift+Space）", () => {
    const log = readLog();
    assert.ok(log, "找不到 waypoint log");
    assert.ok(
      log.includes("register_hotkey ok: Ctrl+Shift+Space"),
      `主 hotkey 註冊失敗或環境不支援。log 末段：\n${log.slice(-1500)}`,
    );
  });

  it("不應出現 register_hotkey failed", () => {
    const log = readLog();
    assert.ok(log);
    const failedLines = log.split("\n").filter((l) => l.includes("register_hotkey failed"));
    assert.equal(failedLines.length, 0, `register_hotkey 有失敗紀錄：\n${failedLines.join("\n")}`);
  });
});
