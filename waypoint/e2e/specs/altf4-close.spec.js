import assert from "node:assert/strict";

async function waitTauriReady() {
  await browser.waitUntil(
    async () =>
      browser.execute(
        () => typeof window.__TAURI_INTERNALS__?.invoke === "function",
      ),
    { timeout: 15_000, timeoutMsg: "Tauri IPC 未就緒" },
  );
}

async function invokeCmd(cmd, args = {}) {
  return browser.executeAsync(
    (c, a, done) => {
      window.__TAURI_INTERNALS__
        .invoke(c, a)
        .then((r) => done({ ok: true, value: r }))
        .catch((e) => done({ ok: false, error: String(e) }));
    },
    cmd,
    args,
  );
}

async function switchToNewWindow(previousHandles) {
  await browser.waitUntil(
    async () => (await browser.getWindowHandles()).length > previousHandles.length,
    { timeout: 10_000, timeoutMsg: "新視窗沒出現" },
  );
  const handles = await browser.getWindowHandles();
  const newHandle = handles.find((h) => !previousHandles.includes(h));
  await browser.switchToWindow(newHandle);
}

// 列表 handle 由 before() 抓住一次。不能用 handles[0]：WebDriver 不保證順序，
// 切到 note 後再呼叫 cmd_close_note_window 會把當前 webview 關掉，executeAsync
// 拿不到結果（act 容器內可重現）。
let listHandle;

async function switchToListWindow() {
  await browser.switchToWindow(listHandle);
}

describe("Alt+F4 關筆記後 list 不自動恢復", () => {
  before(async () => {
    await browser.waitUntil(
      async () => {
        try {
          return (await browser.getPageSource()).includes("WAYPOINT");
        } catch {
          return false;
        }
      },
      { timeout: 20_000, timeoutMsg: "列表視窗未載入" },
    );
    await waitTauriReady();
    const handles = await browser.getWindowHandles();
    listHandle = handles[0];
  });

  it("Alt+F4 等同的視窗關閉後，重開列表不會再自動拉起該筆記", async () => {
    await switchToListWindow();
    const createRes = await invokeCmd("create_note", {
      contextId: null,
      title: "TestAltF4",
    });
    assert.equal(createRes.ok, true, createRes.error);
    const note = createRes.value;

    const before = await browser.getWindowHandles();
    const openRes = await invokeCmd("cmd_open_note_window", {
      noteId: note.id,
      contextId: null,
    });
    assert.equal(openRes.ok, true, openRes.error);
    await switchToNewWindow(before);

    // 模擬 Alt+F4：用 Rust 端 cmd_close_note_window 觸發 win.close()，等同 OS 關閉。
    // 不用 plugin:window|close（webview2 capability 可能擋）。
    // 從 list webview 呼叫，避免從即將被關掉的 note webview 呼叫導致 executeAsync 卡死。
    await switchToListWindow();
    const closeRes = await invokeCmd("cmd_close_note_window", { noteId: note.id });
    assert.equal(closeRes.ok, true, closeRes.error);

    await browser.waitUntil(
      async () => (await browser.getWindowHandles()).length === before.length,
      { timeout: 5_000, timeoutMsg: "Alt+F4 後筆記視窗未關閉" },
    );

    // 列表 toggle：先關列表，再開。每次 toggle 應只開列表，不恢復已 X 過的筆記。
    await switchToListWindow();
    // 收起所有筆記（含可能被 session 紀錄為「打開」的視窗）
    await invokeCmd("cmd_collapse_all", {});

    // 模擬再次叫出列表的場景：emit waypoint://list-shown，list 會 reloadContextAndSession
    await browser.executeAsync((done) => {
      // 直接使用 Tauri internals invoke，避免 bare specifier 在 webview 無法解析
      window.__TAURI_INTERNALS__
        .invoke("plugin:event|emit", { event: "waypoint://list-shown", payload: null })
        .then(() => done(true))
        .catch((e) => done(String(e)));
    });

    // 等候 list 重新處理 session（200ms 緩衝）
    await browser.pause(500);

    // 驗證：handles 仍然只剩列表（沒有自動拉起被關掉的筆記）
    const after = await browser.getWindowHandles();
    assert.equal(
      after.length,
      1,
      `Alt+F4 過的筆記重開列表時被自動拉起，handles=${JSON.stringify(after)}`,
    );
  });
});
