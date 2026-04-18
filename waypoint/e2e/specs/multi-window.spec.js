import assert from "node:assert/strict";

// 等 Tauri IPC 可用（__TAURI_INTERNALS__.invoke 存在）
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

async function assertNonBlankAndContains(expectedSubstrings) {
  await browser.waitUntil(
    async () => {
      try {
        const txt = await browser.execute(() => document.body.innerText.trim());
        return txt.length > 0;
      } catch {
        return false;
      }
    },
    { timeout: 15_000, timeoutMsg: "新視窗 body 空白（白屏）" },
  );
  const src = await browser.getPageSource();
  assert.ok(
    expectedSubstrings.some((s) => src.includes(s)),
    `視窗內容不含預期字串 ${JSON.stringify(expectedSubstrings)}`,
  );
}

describe("help / settings / note 視窗非白屏", () => {
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

  async function backToList() {
    const handles = await browser.getWindowHandles();
    await browser.switchToWindow(handles[0]);
  }

  it("說明視窗：cmd_open_help 後有內容", async () => {
    await backToList();
    const before = await browser.getWindowHandles();
    const res = await invokeCmd("cmd_open_help");
    assert.ok(res.ok, `cmd_open_help 失敗: ${res.error}`);
    await switchToNewWindow(before);
    await assertNonBlankAndContains(["快捷鍵邏輯", "使用說明"]);
  });

  it("設定視窗：cmd_open_settings 後有內容", async () => {
    await backToList();
    const before = await browser.getWindowHandles();
    const res = await invokeCmd("cmd_open_settings");
    assert.ok(res.ok, `cmd_open_settings 失敗: ${res.error}`);
    await switchToNewWindow(before);
    await assertNonBlankAndContains(["Waypoint — 設定", "快捷鍵"]);
  });

  it("筆記視窗：建新筆記並開啟後有內容", async () => {
    await backToList();

    const createRes = await invokeCmd("create_note", {
      contextId: null,
      title: "E2E 測試",
    });
    assert.ok(createRes.ok, `create_note 失敗: ${createRes.error}`);
    const noteId = createRes.value.id;

    const before = await browser.getWindowHandles();
    const openRes = await invokeCmd("cmd_open_note_window", {
      noteId,
      contextId: null,
    });
    assert.ok(openRes.ok, `cmd_open_note_window 失敗: ${openRes.error}`);
    await switchToNewWindow(before);
    await assertNonBlankAndContains([
      "E2E 測試",
      "editor-area",
      "Markdown",
    ]);
  });
});
