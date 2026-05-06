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

async function switchToNewWindow(prev) {
  await browser.waitUntil(
    async () => (await browser.getWindowHandles()).length > prev.length,
    { timeout: 10_000 },
  );
  const handles = await browser.getWindowHandles();
  const newHandle = handles.find((h) => !prev.includes(h));
  await browser.switchToWindow(newHandle);
}

describe("穿透模式 chrome 隱藏", () => {
  before(async () => {
    await browser.waitUntil(
      async () => (await browser.getPageSource()).includes("WAYPOINT"),
      { timeout: 20_000 },
    );
    await waitTauriReady();
    listHandle = (await browser.getWindowHandles())[0];
  });

  it("toggle on：note titlebar 隱藏；toggle off：還原", async () => {
    await browser.switchToWindow(listHandle);
    const created = await invokeCmd("create_note", { contextId: null, title: "PT-Chrome" });
    assert.equal(created.ok, true);
    const noteId = created.value.id;

    const before = await browser.getWindowHandles();
    await invokeCmd("cmd_open_note_window", { noteId, contextId: null });
    await switchToNewWindow(before);

    // 在 note webview：driving-titlebar 一開始可見
    let titlebarCount = await browser.execute(
      () => document.querySelectorAll('.draggable-titlebar').length,
    );
    assert.equal(titlebarCount, 1, "進穿透前 titlebar 應存在");

    // 從 list 觸發 global toggle on
    await browser.switchToWindow(listHandle);
    await invokeCmd("cmd_toggle_passthrough_global", {});
    await browser.pause(400);

    const handles = await browser.getWindowHandles();
    const noteHandle = handles.find((h) => h !== listHandle);
    await browser.switchToWindow(noteHandle);
    titlebarCount = await browser.execute(
      () => document.querySelectorAll('.draggable-titlebar').length,
    );
    assert.equal(titlebarCount, 0, "穿透 on 後 titlebar 應消失");

    // toggle off → titlebar 還原
    await browser.switchToWindow(listHandle);
    await invokeCmd("cmd_toggle_passthrough_global", {});
    await browser.pause(400);

    await browser.switchToWindow(noteHandle);
    titlebarCount = await browser.execute(
      () => document.querySelectorAll('.draggable-titlebar').length,
    );
    assert.equal(titlebarCount, 1, "穿透 off 後 titlebar 應還原");

    // cleanup
    await browser.switchToWindow(listHandle);
    await invokeCmd("cmd_close_note_window", { noteId });
    await invokeCmd("delete_note", { contextId: null, noteId });
  });
});
