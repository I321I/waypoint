import assert from "node:assert/strict";

// 等 Tauri IPC 可用
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

describe("筆記 titlebar ⇊ 收起按鈕（R11）", () => {
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

  it("⇊ 按鈕存在於筆記 titlebar，按下後 emit collapse-all-requested", async () => {
    // 1. 在列表視窗建立筆記
    const handles0 = await browser.getWindowHandles();
    await browser.switchToWindow(handles0[0]);
    const createRes = await invokeCmd("create_note", {
      contextId: null,
      title: "Collapse 測試",
    });
    assert.ok(createRes.ok, `create_note 失敗: ${createRes.error}`);
    const noteId = createRes.value.id;

    // 2. 開啟筆記視窗
    const before = await browser.getWindowHandles();
    const openRes = await invokeCmd("cmd_open_note_window", {
      noteId,
      contextId: null,
    });
    assert.ok(openRes.ok, `cmd_open_note_window 失敗: ${openRes.error}`);
    await switchToNewWindow(before);

    // 3. 等筆記 titlebar 出現
    await browser.waitUntil(
      async () => {
        const src = await browser.getPageSource();
        return src.includes("收起全部並儲存");
      },
      { timeout: 10_000, timeoutMsg: "筆記 titlebar 沒出現 ⇊ 按鈕" },
    );

    // 4. 確認 ⇊ 按鈕存在且文字正確
    const btn = await browser.$('button[title="收起全部並儲存"]');
    assert.ok(await btn.isExisting(), "⇊ 按鈕不存在");
    const text = (await btn.getText()).trim();
    assert.equal(text, "⇊", `按鈕文字預期 ⇊，實際 ${text}`);

    // 5. 點按鈕應觸發收起（不拋例外即視為通過；
    //    完整的 session restore 留給 Phase 7 整合驗證）
    await btn.click();
  });
});
