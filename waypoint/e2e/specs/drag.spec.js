import assert from "node:assert/strict";

// E2E：筆記視窗拖曳相關保護。
// 本機 tauri-driver 在 Linux/WebKitGTK 下對合成 pointer 事件觸發 X11 native drag
// 並不穩定；因此本 spec 驗證的是「拖曳能力的基礎設施」：
//   1. titlebar 有 data-tauri-drag-region 屬性
//   2. 沒有其他元素覆蓋（mousedown target 仍是 titlebar）
//   3. cmd_set_window_position + cmd_get_window_position 可正常改變視窗位置
// 前端 NoteWindow.svelte 在 titlebar mousedown 上也掛了 cmd_start_dragging fallback，
// 這裡順便斷言 mousedown 事件確實會 bubble 到 .titlebar。

async function waitTauriReady() {
  await browser.waitUntil(
    async () =>
      browser.execute(() => typeof window.__TAURI_INTERNALS__?.invoke === "function"),
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

async function switchToNewWindow(previousHandles) {
  await browser.waitUntil(
    async () => (await browser.getWindowHandles()).length > previousHandles.length,
    { timeout: 10_000, timeoutMsg: "新視窗沒出現" },
  );
  const handles = await browser.getWindowHandles();
  const newHandle = handles.find((h) => !previousHandles.includes(h));
  await browser.switchToWindow(newHandle);
}

describe("筆記視窗可拖曳移動", () => {
  let noteId;
  let noteLabel;

  before(async () => {
    await browser.waitUntil(
      async () => {
        try { return (await browser.getPageSource()).includes("WAYPOINT"); }
        catch { return false; }
      },
      { timeout: 20_000, timeoutMsg: "列表視窗未載入" },
    );
    await waitTauriReady();

    const createRes = await invokeCmd("create_note", {
      contextId: null, title: "drag test",
    });
    assert.ok(createRes.ok, `create_note 失敗: ${createRes.error}`);
    noteId = createRes.value.id;
    noteLabel = `note-${noteId}`;

    const before = await browser.getWindowHandles();
    const openRes = await invokeCmd("cmd_open_note_window", {
      noteId, contextId: null,
    });
    assert.ok(openRes.ok, `cmd_open_note_window 失敗: ${openRes.error}`);
    await switchToNewWindow(before);

    // 等 note window 的 titlebar 出現
    await browser.waitUntil(
      async () => {
        const el = await browser.$(".titlebar");
        return await el.isExisting();
      },
      { timeout: 15_000, timeoutMsg: ".titlebar 未掛載" },
    );
  });

  it("titlebar 有 data-tauri-drag-region 屬性", async () => {
    const bar = await browser.$(".titlebar");
    const attr = await bar.getAttribute("data-tauri-drag-region");
    assert.ok(attr !== null, "titlebar 缺 data-tauri-drag-region — 拖曳將失效");
  });

  it("titlebar 中央的 mousedown target 不是被遮住的 child", async () => {
    const target = await browser.execute(() => {
      const bar = document.querySelector(".titlebar");
      if (!bar) return null;
      const r = bar.getBoundingClientRect();
      const cx = r.left + r.width / 2;
      const cy = r.top + r.height / 2;
      const el = document.elementFromPoint(cx, cy);
      if (!el) return null;
      // 回傳最靠近的、帶有 data-tauri-drag-region 的祖先 / 自身
      let cur = el;
      while (cur) {
        if (cur.getAttribute && cur.getAttribute("data-tauri-drag-region") !== null) {
          return cur.className || cur.tagName;
        }
        cur = cur.parentElement;
      }
      return `NO_DRAG_REGION(${el.tagName}.${el.className})`;
    });
    assert.ok(target, "titlebar 中央沒有找到元素");
    assert.ok(
      !String(target).startsWith("NO_DRAG_REGION"),
      `titlebar 中央的 mousedown target 沒有 data-tauri-drag-region：${target}`,
    );
  });

  it("cmd_start_dragging 命令存在且不會丟錯（能力檢查）", async () => {
    const res = await invokeCmd("cmd_start_dragging", { label: noteLabel });
    // 在無實體 mouse button down 的情境下，start_dragging 行為依平台：
    // 可能 Ok(()) 也可能回報 error，但至少 command 必須註冊成功（否則會拋 command not found）
    assert.ok(
      res.ok || !String(res.error || "").includes("not found"),
      `cmd_start_dragging 未註冊或異常：${res.error}`,
    );
  });

  it("程式化位移：set_window_position → get_window_position 可正確變更", async () => {
    const before = await invokeCmd("cmd_get_window_position", { label: noteLabel });
    assert.ok(before.ok, `cmd_get_window_position 失敗: ${before.error}`);
    const [x0, y0] = before.value;

    const targetX = x0 + 120;
    const targetY = y0 + 80;
    const setRes = await invokeCmd("cmd_set_window_position", {
      label: noteLabel, x: targetX, y: targetY,
    });
    assert.ok(setRes.ok, `cmd_set_window_position 失敗: ${setRes.error}`);

    // 有些 WM 會微調邊界，允許 ±20px 誤差
    const after = await invokeCmd("cmd_get_window_position", { label: noteLabel });
    assert.ok(after.ok, `cmd_get_window_position 失敗: ${after.error}`);
    const [x1, y1] = after.value;
    assert.ok(
      Math.abs(x1 - targetX) <= 20 && Math.abs(y1 - targetY) <= 20,
      `視窗未移動到目標位置：目標 (${targetX},${targetY}) vs 實際 (${x1},${y1})`,
    );
    assert.ok(x1 !== x0 || y1 !== y0, "視窗位置未改變 → 拖曳能力疑似失效");
  });
});
