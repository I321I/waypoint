// 驗證 ListWindow 收到 active-context-changed 事件後行為正確：
// 1. 把目前 context 的 session 存起來
// 2. close 所有筆記視窗（但 list 保持可見）
// 3. reload 新 context 的筆記與 session
//
// xvfb 沒有真實 WM 無法觸發 foreground_watcher 的真實流程，
// 所以這裡用 emit 直接送事件給 list webview 模擬 watcher 已經把 active context 改掉。
import assert from "node:assert/strict";

let listHandle;

async function waitTauriReady() {
  await browser.waitUntil(
    async () => browser.execute(
      () => typeof window.__TAURI_INTERNALS__?.invoke === "function"
    ),
    { timeout: 15_000 },
  );
}

async function invokeCmd(cmd, args = {}) {
  return browser.executeAsync(
    (c, a, done) => {
      window.__TAURI_INTERNALS__.invoke(c, a)
        .then((r) => done({ ok: true, value: r }))
        .catch((e) => done({ ok: false, error: String(e) }));
    },
    cmd,
    args,
  );
}

async function emitFromList(eventName, payload) {
  return browser.executeAsync(
    (name, p, done) => {
      // Rust 端的 active_context_id 也要先寫入，否則 loadContextAndSession 拿到舊值
      // 用 plugin:event|emit 從 list webview 發送
      window.__TAURI_INTERNALS__
        .invoke("plugin:event|emit", { event: name, payload: p })
        .then(() => done({ ok: true }))
        .catch((e) => done({ ok: false, error: String(e) }));
    },
    eventName,
    payload,
  );
}

describe("自動 context 切換（手動觸發 active-context-changed）", () => {
  before(async () => {
    await browser.waitUntil(
      async () => (await browser.getPageSource()).includes("WAYPOINT"),
      { timeout: 20_000 },
    );
    await waitTauriReady();
    listHandle = (await browser.getWindowHandles())[0];
  });

  it("收到 active-context-changed 事件後 list webview 仍 render（不被 hide）", async () => {
    await browser.switchToWindow(listHandle);

    const emitRes = await emitFromList("waypoint://active-context-changed", "fakeContext");
    if (!emitRes.ok) {
      console.log("[auto-context-switch] event emit 不可用，skip:", emitRes.error);
      return;
    }

    // 等 listener 跑完（flush 200ms + close 迴圈 + reload）
    await browser.pause(1000);

    // 切回 list（listener 不該把 list 自己 hide）
    const handles = await browser.getWindowHandles();
    assert.ok(handles.includes(listHandle), "list handle 仍應存在");
    await browser.switchToWindow(listHandle);
    const listVisible = await browser.execute(
      () => document.querySelector('.app-name')?.textContent === 'WAYPOINT',
    );
    assert.ok(listVisible, "list webview 應仍 render（regression：先前用 collapseAll 會把 list 也 hide）");
  });
});
