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
  // 列表視窗為第一個
  await browser.switchToWindow(handles[0]);
}

describe("筆記內容關閉後仍持久化", () => {
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

  it("輸入內容 → 點 ✕ 關閉 → 重新讀取，內容仍在", async () => {
    // 1. 建立筆記
    await switchToListWindow();
    const createRes = await invokeCmd("create_note", {
      contextId: null,
      title: "Persistence 測試",
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

    // 3. 等 tiptap editor mount + 暴露 window.__waypointTiptapEditor
    await browser.waitUntil(
      async () => {
        return browser.execute(() => !!(window).__waypointTiptapEditor);
      },
      { timeout: 10_000, timeoutMsg: "編輯器未掛載 / __waypointTiptapEditor 未暴露" },
    );

    // 4. 透過 tiptap commands 寫入內容（避開 WebDriver 對 contenteditable 鍵盤模擬的跨平台不一致）
    const sentinel = "PERSIST-" + Date.now();
    await browser.execute((text) => {
      const ed = (window).__waypointTiptapEditor;
      ed.commands.insertContent(text);
    }, sentinel);

    // 5. 等內容寫入 editor DOM
    const editor = await browser.$(".tiptap-editor");
    await browser.waitUntil(
      async () => {
        const html = await editor.getHTML();
        return html.includes(sentinel);
      },
      { timeout: 5_000, timeoutMsg: "輸入文字未顯示" },
    );

    // 6. 點 ✕ 關閉按鈕（觸發 flushPendingSave）
    const closeBtn = await browser.$('button[title="儲存並關閉"]');
    assert.ok(await closeBtn.isExisting(), "✕ 按鈕不存在");

    const beforeClose = await browser.getWindowHandles();
    await closeBtn.click();

    // 7. 等視窗消失
    await browser.waitUntil(
      async () => (await browser.getWindowHandles()).length < beforeClose.length,
      { timeout: 10_000, timeoutMsg: "筆記視窗未關閉" },
    );

    // 8. 切回列表視窗，read_note 確認內容已寫入
    await switchToListWindow();
    const readRes = await invokeCmd("read_note", { contextId: null, noteId });
    assert.ok(readRes.ok, `read_note 失敗: ${readRes.error}`);
    assert.ok(
      readRes.value.content.includes(sentinel),
      `關閉後 read_note 內容遺失：實際 = ${JSON.stringify(readRes.value.content)}`,
    );
  });

  it("關閉後重新開啟，markdown 會 render（# Heading 變 <h1>）", async () => {
    // 1. 建立並寫入帶 markdown 的內容
    await switchToListWindow();
    const createRes = await invokeCmd("create_note", {
      contextId: null,
      title: "Markdown render 測試",
    });
    assert.ok(createRes.ok, `create_note 失敗: ${createRes.error}`);
    const noteId = createRes.value.id;

    // 直接寫 markdown 到 disk（避開 Editor 路徑，模擬 saved file）
    const mdContent = "# Heading\n\n**bold** text";
    const saveRes = await invokeCmd("save_content", {
      contextId: null,
      noteId,
      content: mdContent,
    });
    assert.ok(saveRes.ok, `save_content 失敗: ${saveRes.error}`);

    // 2. 開啟筆記視窗
    const before = await browser.getWindowHandles();
    const openRes = await invokeCmd("cmd_open_note_window", {
      noteId,
      contextId: null,
    });
    assert.ok(openRes.ok, `cmd_open_note_window 失敗: ${openRes.error}`);
    await switchToNewWindow(before);

    // 3. 等 editor mount + setContent
    await browser.waitUntil(
      async () => browser.execute(() => !!(window).__waypointTiptapEditor),
      { timeout: 10_000, timeoutMsg: "編輯器未掛載" },
    );

    // 4. editor 內應該有 <h1> 與 <strong> 元素，而不是純文字 # 與 **
    const editor = await browser.$(".tiptap-editor");
    const html = await editor.getHTML();
    assert.ok(
      /<h1[^>]*>Heading<\/h1>/i.test(html),
      `markdown 沒被 render — 找不到 <h1>Heading</h1>，實際 HTML：${html}`,
    );
    assert.ok(
      /<strong>bold<\/strong>/i.test(html),
      `markdown 沒被 render — 找不到 <strong>bold</strong>，實際 HTML：${html}`,
    );
    assert.ok(
      !html.includes("# Heading"),
      `markdown 原貌仍出現在 HTML（應已 render 掉），實際：${html}`,
    );
  });
});
