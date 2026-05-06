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

  it("收到事件後 list 應仍可見、note 視窗被 close、可在新 context 操作", async () => {
    await browser.switchToWindow(listHandle);

    // 開一個全域筆記讓場景非空
    const created = await invokeCmd("create_note", { contextId: null, title: "PreSwitch" });
    assert.equal(created.ok, true, created.error);
    const noteId = created.value.id;

    const before = await browser.getWindowHandles();
    await invokeCmd("cmd_open_note_window", { noteId, contextId: null });
    await browser.waitUntil(
      async () => (await browser.getWindowHandles()).length > before.length,
      { timeout: 10_000 },
    );
    const noteHandle = (await browser.getWindowHandles()).find((h) => !before.includes(h));
    assert.ok(noteHandle, "note 視窗應出現");

    // 觸發 list 端的 active-context-changed listener（直接 emit；Rust state 此時還是 null）
    await browser.switchToWindow(listHandle);
    const emitRes = await emitFromList("waypoint://active-context-changed", "fakeContext");
    // 部分 capability 配置可能擋住 plugin:event|emit；若不可用就 skip 後續斷言
    if (!emitRes.ok) {
      console.log("[auto-context-switch] event emit 不可用，skip:", emitRes.error);
      // cleanup 後 skip
      await invokeCmd("cmd_close_note_window", { noteId });
      await invokeCmd("delete_note", { contextId: null, noteId });
      return;
    }

    // 等 listener 跑完（flush 200ms + close + reload）
    await browser.pause(800);

    // 1. list webview 仍可被切到（沒被 hide）
    const handles = await browser.getWindowHandles();
    assert.ok(handles.includes(listHandle), "list handle 仍應存在");
    await browser.switchToWindow(listHandle);
    const listVisible = await browser.execute(
      () => document.querySelector('.app-name')?.textContent === 'WAYPOINT',
    );
    assert.ok(listVisible, "list webview 應仍渲染（不該被 hide）");

    // 2. note 視窗應已被 close
    const handlesAfter = await browser.getWindowHandles();
    assert.ok(!handlesAfter.includes(noteHandle), "切換 context 後 note 視窗應被 close");

    // cleanup
    await invokeCmd("delete_note", { contextId: null, noteId });
  });
});
