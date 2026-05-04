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

async function switchToListWindow() {
  const handles = await browser.getWindowHandles();
  await browser.switchToWindow(handles[0]);
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

    // 模擬 Alt+F4：在筆記 webview 觸發 close-requested。
    // NoteWindow.svelte 攔截 close-requested 並呼叫 handleClose（emit note-closed + close window）。
    await browser.executeAsync((done) => {
      // 透過 Tauri window API：getCurrentWindow().close() 會觸發 close-requested
      import("@tauri-apps/api/window")
        .then(({ getCurrentWindow }) => getCurrentWindow().close())
        .then(() => done(true))
        .catch((e) => done(String(e)));
    });

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
      import("@tauri-apps/api/event")
        .then(({ emit }) => emit("waypoint://list-shown"))
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
