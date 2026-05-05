// 驗證「開機自動啟動」設定切換時，OS 端 desktop file 真的被建立 / 刪除。
// 僅 Linux 支援（is_autostart_supported 在 Win/Mac 為 false）。
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";

function autostartFilePath() {
  const xdg = process.env.XDG_CONFIG_HOME;
  const base = xdg && xdg.length > 0
    ? xdg
    : path.join(os.homedir(), ".config");
  return path.join(base, "autostart", "waypoint.desktop");
}

async function invoke(cmd, args = {}) {
  return browser.executeAsync(
    (cmd, args, done) => {
      const t = window.__TAURI__ || window.__TAURI_INTERNALS__;
      const inv = window.__TAURI__?.core?.invoke || window.__TAURI_INTERNALS__?.invoke;
      if (!inv) {
        done({ error: "no tauri invoke available" });
        return;
      }
      inv(cmd, args).then((r) => done({ ok: r ?? null })).catch((e) => done({ error: String(e) }));
    },
    cmd,
    args,
  );
}

describe("autostart 設定切換 (Linux only)", function () {
  before(async function () {
    if (process.platform !== "linux") return this.skip();
    await browser.waitUntil(
      async () => (await browser.getPageSource()).includes("WAYPOINT"),
      { timeout: 20_000 },
    );

    const sup = await invoke("is_autostart_supported");
    if (!sup || sup.error || sup.ok !== true) return this.skip();
  });

  it("set_autostart(true) 後 .desktop 檔存在；set_autostart(false) 後刪除", async () => {
    const filePath = autostartFilePath();

    // 起始狀態清乾淨
    try { fs.rmSync(filePath, { force: true }); } catch {}

    let r = await invoke("set_autostart", { enabled: true });
    assert.ok(!r.error, `set_autostart(true) 失敗：${r.error}`);
    assert.ok(fs.existsSync(filePath), `啟用後應存在 ${filePath}`);

    const content = fs.readFileSync(filePath, "utf8");
    assert.ok(content.includes("[Desktop Entry]"), "desktop file 內容不正確");
    assert.ok(content.includes("Name=Waypoint"));

    let g = await invoke("get_autostart");
    assert.equal(g.ok, true, "get_autostart 應回傳 true");

    r = await invoke("set_autostart", { enabled: false });
    assert.ok(!r.error, `set_autostart(false) 失敗：${r.error}`);
    assert.ok(!fs.existsSync(filePath), "停用後 .desktop 檔應已刪除");

    g = await invoke("get_autostart");
    assert.equal(g.ok, false, "get_autostart 應回傳 false");
  });
});
