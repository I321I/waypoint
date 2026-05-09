// 真實驗證 foreground_watcher：spawn xterm，用 xdotool 切焦點到它，
// 確認 watcher 從 X11 _NET_ACTIVE_WINDOW 抓到變化、emit active-context-changed。
//
// 需要：xvfb + openbox（提供 EWMH _NET_ACTIVE_WINDOW 行為）+ xterm + xdotool
// 跑法：CI 已在 e2e-linux.yml 把 openbox 起來；本機 act 也吃同一份 yml
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { readLog } from "../log-path.js";

function hasBin(name) {
  return spawnSync(name, ["--version"], { stdio: "ignore" }).status === 0
      || spawnSync(name, ["-v"], { stdio: "ignore" }).status === 0;
}

function checkWmRunning() {
  // openbox 啟動後 root window 會有 _NET_SUPPORTING_WM_CHECK 屬性
  const r = spawnSync("xprop", ["-root", "_NET_SUPPORTING_WM_CHECK"], { encoding: "utf8" });
  return r.status === 0 && /window id #/.test(r.stdout);
}

describe("foreground_watcher 偵測 OS 焦點變化（需 WM）", function () {
  let xtermProc;

  before(async function () {
    if (process.platform !== "linux") return this.skip();
    if (!process.env.DISPLAY) return this.skip();
    for (const bin of ["xterm", "xdotool", "xprop"]) {
      if (!hasBin(bin)) {
        console.log(`[fg-watcher] missing ${bin}; skip`);
        return this.skip();
      }
    }
    if (!checkWmRunning()) {
      console.log("[fg-watcher] _NET_SUPPORTING_WM_CHECK 不存在（沒 WM）；skip");
      return this.skip();
    }

    await browser.waitUntil(
      async () => (await browser.getPageSource()).includes("WAYPOINT"),
      { timeout: 20_000 },
    );

    // 給 watcher 時間 startup 後再開 xterm
    await new Promise((r) => setTimeout(r, 800));
  });

  after(() => {
    if (xtermProc && !xtermProc.killed) {
      try { xtermProc.kill("SIGKILL"); } catch {}
    }
  });

  it("xterm 取得焦點後 watcher 應 emit active-context-changed for xterm", async () => {
    const before = readLog();

    // spawn xterm，detached 讓它不依附本進程；-T 設 title 方便 xdotool 找
    xtermProc = spawn("xterm", ["-T", "WaypointWatcherTest", "-e", "sleep 60"], {
      stdio: "ignore",
      detached: true,
      env: { ...process.env },
    });
    xtermProc.unref();

    // 等 xterm 視窗出現
    let wid = "";
    const findDeadline = Date.now() + 8000;
    while (Date.now() < findDeadline) {
      const r = spawnSync(
        "xdotool",
        ["search", "--name", "WaypointWatcherTest"],
        { encoding: "utf8" },
      );
      const found = r.stdout.trim().split(/\s+/).filter(Boolean);
      if (found.length > 0) { wid = found[0]; break; }
      await new Promise((r) => setTimeout(r, 200));
    }
    assert.ok(wid, "找不到 xterm window id（xterm 沒成功 spawn 或 X11 未起來）");

    // 用 xdotool windowactivate（依賴 WM 處理）
    const actRes = spawnSync("xdotool", ["windowactivate", "--sync", wid], {
      stdio: "ignore",
    });
    assert.equal(actRes.status, 0, "xdotool windowactivate 失敗（WM 未啟動？）");

    // 等 watcher 至少跑兩輪 200ms poll + emit + log flush
    let after = "";
    const deadline = Date.now() + 5000;
    let firedNew = false;
    while (Date.now() < deadline) {
      after = readLog();
      const newPart = after.slice(before.length);
      if (/active-context-changed: emit ctx_id=xterm/.test(newPart)) {
        firedNew = true;
        break;
      }
      await new Promise((r) => setTimeout(r, 200));
    }

    assert.ok(
      firedNew,
      `watcher 應該偵測到 xterm 焦點變化並 emit ctx_id=xterm。\n新增 log 段：\n${after.slice(before.length).slice(-2000)}`,
    );
  });

  it("切回 waypoint 視窗時不應 emit ctx_id=waypoint（PID 過濾自家）", async () => {
    const before = readLog();
    // tauri-driver session 對應 waypoint binary 的 list 視窗。
    // 用 xdotool search waypoint title / class 嘗試找它的 window
    const search = spawnSync(
      "xdotool",
      ["search", "--name", "Waypoint"],
      { encoding: "utf8" },
    );
    const wids = search.stdout.trim().split(/\s+/).filter(Boolean);
    if (wids.length === 0) {
      console.log("[fg-watcher] 找不到 Waypoint 視窗 id，skip 第二段");
      return;
    }
    spawnSync("xdotool", ["windowactivate", "--sync", wids[0]], { stdio: "ignore" });
    await new Promise((r) => setTimeout(r, 800));

    const after = readLog();
    const newPart = after.slice(before.length);
    assert.ok(
      !/active-context-changed: emit ctx_id=waypoint/.test(newPart),
      `切到 waypoint 自家視窗不該觸發 emit ctx_id=waypoint。新 log:\n${newPart.slice(-1500)}`,
    );
  });
});
