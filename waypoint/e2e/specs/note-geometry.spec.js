import assert from "node:assert/strict";

let listHandle;

async function waitTauriReady() {
  await browser.waitUntil(
    async () => browser.execute(
      () => typeof window.__TAURI_INTERNALS__?.invoke === "function"
    ),
    { timeout: 15_000, timeoutMsg: "Tauri IPC 未就緒" },
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

async function switchToNewWindow(prev) {
  await browser.waitUntil(
    async () => (await browser.getWindowHandles()).length > prev.length,
    { timeout: 10_000, timeoutMsg: "新視窗沒出現" },
  );
  const handles = await browser.getWindowHandles();
  const newHandle = handles.find((h) => !prev.includes(h));
  await browser.switchToWindow(newHandle);
}

async function switchToList() {
  await browser.switchToWindow(listHandle);
}

describe("筆記視窗幾何即時記憶", () => {
  before(async () => {
    await browser.waitUntil(
      async () => (await browser.getPageSource()).includes("WAYPOINT"),
      { timeout: 20_000 },
    );
    await waitTauriReady();
    listHandle = (await browser.getWindowHandles())[0];
  });

  it("移動位置後關閉，重開應還原到新位置", async () => {
    await switchToList();
    const created = await invokeCmd("create_note", { contextId: null, title: "GeoTest" });
    assert.equal(created.ok, true, created.error);
    const noteId = created.value.id;

    const before = await browser.getWindowHandles();
    const opened = await invokeCmd("cmd_open_note_window", { noteId, contextId: null });
    assert.equal(opened.ok, true, opened.error);
    await switchToNewWindow(before);

    // 從 list 設位置（避開從 note webview 操作）
    await switchToList();
    const setPos = await invokeCmd("cmd_set_window_position", { label: `note-${noteId}`, x: 150, y: 220 });
    assert.equal(setPos.ok, true, setPos.error);

    // 等 debounce 500ms + io
    await browser.pause(900);

    // 從 list 關掉視窗
    await invokeCmd("cmd_close_note_window", { noteId });
    await browser.waitUntil(
      async () => (await browser.getWindowHandles()).length === before.length,
      { timeout: 5_000 },
    );

    // 重開
    const before2 = await browser.getWindowHandles();
    const opened2 = await invokeCmd("cmd_open_note_window", { noteId, contextId: null });
    assert.equal(opened2.ok, true, opened2.error);
    await switchToNewWindow(before2);

    await switchToList();
    const pos = await invokeCmd("cmd_get_window_position", { label: `note-${noteId}` });
    assert.equal(pos.ok, true, pos.error);
    const [x, y] = pos.value;
    // 容忍 WM 加 deco/offset 的小誤差（WebKitGTK xvfb 有 ±1 offset）
    assert.ok(Math.abs(x - 150) <= 5, `重開後 x 預期 ~150，實際 ${x}`);
    assert.ok(Math.abs(y - 220) <= 5, `重開後 y 預期 ~220，實際 ${y}`);

    // cleanup
    await invokeCmd("cmd_close_note_window", { noteId });
    await invokeCmd("delete_note", { contextId: null, noteId });
  });
});
