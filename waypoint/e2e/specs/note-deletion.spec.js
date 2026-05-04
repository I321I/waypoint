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

describe("刪除筆記同步生命週期", () => {
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

  it("從列表刪除筆記 → 已開啟的筆記視窗自動關閉", async () => {
    await switchToListWindow();
    const createRes = await invokeCmd("create_note", {
      contextId: null,
      title: "TestDelFromList",
    });
    assert.equal(createRes.ok, true, createRes.error);
    const note = createRes.value;

    // 開啟筆記視窗
    const before = await browser.getWindowHandles();
    const openRes = await invokeCmd("cmd_open_note_window", {
      noteId: note.id,
      contextId: null,
    });
    assert.equal(openRes.ok, true, openRes.error);
    await switchToNewWindow(before);

    // 切回列表，刪除筆記
    await switchToListWindow();
    const delRes = await invokeCmd("delete_note", {
      contextId: null,
      noteId: note.id,
    });
    assert.equal(delRes.ok, true, delRes.error);

    // 等待筆記視窗關閉
    await browser.waitUntil(
      async () => (await browser.getWindowHandles()).length === before.length,
      { timeout: 5_000, timeoutMsg: "刪除後筆記視窗未自動關閉" },
    );

    // 列表已不再包含此筆記
    const listRes = await invokeCmd("list_notes", { contextId: null });
    assert.equal(listRes.ok, true, listRes.error);
    const ids = listRes.value.map((n) => n.id);
    assert.equal(ids.includes(note.id), false, "刪除後 list_notes 仍含該筆記");
  });

  it("從筆記設定面板刪除筆記 → 列表同步移除", async () => {
    await switchToListWindow();
    const createRes = await invokeCmd("create_note", {
      contextId: null,
      title: "TestDelFromNote",
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

    // 直接走 backend delete_note（與 SettingsPanel 的「刪除此筆記」呼叫等效）
    const delRes = await invokeCmd("delete_note", {
      contextId: null,
      noteId: note.id,
    });
    assert.equal(delRes.ok, true, delRes.error);

    await browser.waitUntil(
      async () => (await browser.getWindowHandles()).length === before.length,
      { timeout: 5_000, timeoutMsg: "從筆記內刪除後筆記視窗未關閉" },
    );

    await switchToListWindow();
    const listRes = await invokeCmd("list_notes", { contextId: null });
    assert.equal(listRes.ok, true, listRes.error);
    const ids = listRes.value.map((n) => n.id);
    assert.equal(ids.includes(note.id), false, "從筆記內刪除後列表仍含該筆記");
  });
});
