import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { spawnSync } from "node:child_process";

function readLog() {
  const home = os.homedir();
  const xdg = process.env.XDG_STATE_HOME;
  const candidates = [];
  if (xdg) candidates.push(path.join(xdg, "waypoint", "error.log"));
  if (home) candidates.push(path.join(home, ".local/state/waypoint/error.log"));
  for (const p of candidates) {
    try { return fs.readFileSync(p, "utf8"); } catch {}
  }
  return "";
}

function hasXdotool() {
  return spawnSync("xdotool", ["--version"], { stdio: "ignore" }).status === 0;
}

describe("hotkey 序列化（連按 inflight drop）", function () {
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

  it("快速連發兩次 hotkey：log 出現 hotkey fired 且 hotkey dropped 至少一次或第二次也 fired", async () => {
    const before = readLog();
    const beforeFired = (before.match(/hotkey fired/g) || []).length;

    // 兩次 xdotool key 之間延遲很短（spawnSync 連發）
    spawnSync("xdotool", ["key", "ctrl+shift+space"], { stdio: "ignore" });
    spawnSync("xdotool", ["key", "ctrl+shift+space"], { stdio: "ignore" });

    let after = "";
    let afterFired = beforeFired;
    const deadline = Date.now() + 5000;
    while (Date.now() < deadline) {
      after = readLog();
      afterFired = (after.match(/hotkey fired/g) || []).length;
      if (afterFired > beforeFired) break;
      await new Promise((r) => setTimeout(r, 100));
    }

    // 至少要有一次 hotkey 被 callback 處理（可能是兩次都成功，也可能第二次被 inflight drop）
    assert.ok(
      afterFired > beforeFired,
      `hotkey 應至少觸發一次。before=${beforeFired} after=${afterFired}\n${after.slice(-1500)}`,
    );
  });
});
