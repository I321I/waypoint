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

describe("Help / Settings titlebar 拖曳屬性（R12）", () => {
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

  it("Help 視窗 .titlebar 具備 data-tauri-drag-region 屬性", async () => {
    await backToList();
    const before = await browser.getWindowHandles();
    const res = await invokeCmd("cmd_open_help");
    assert.ok(res.ok, `cmd_open_help 失敗: ${res.error}`);
    await switchToNewWindow(before);

    await browser.waitUntil(
      async () => {
        const src = await browser.getPageSource();
        return src.includes("使用說明");
      },
      { timeout: 10_000, timeoutMsg: "Help 視窗沒載入" },
    );
    const has = await browser.execute(() => {
      const tb = document.querySelector(".titlebar");
      return !!tb && tb.hasAttribute("data-tauri-drag-region");
    });
    assert.ok(has, "Help titlebar 缺少 data-tauri-drag-region");
  });

  it("Settings 視窗 .titlebar 具備 data-tauri-drag-region 屬性", async () => {
    await backToList();
    const before = await browser.getWindowHandles();
    const res = await invokeCmd("cmd_open_settings");
    assert.ok(res.ok, `cmd_open_settings 失敗: ${res.error}`);
    await switchToNewWindow(before);

    await browser.waitUntil(
      async () => {
        const src = await browser.getPageSource();
        return src.includes("Waypoint — 設定");
      },
      { timeout: 10_000, timeoutMsg: "Settings 視窗沒載入" },
    );
    const has = await browser.execute(() => {
      const tb = document.querySelector(".titlebar");
      return !!tb && tb.hasAttribute("data-tauri-drag-region");
    });
    assert.ok(has, "Settings titlebar 缺少 data-tauri-drag-region");
  });
});
