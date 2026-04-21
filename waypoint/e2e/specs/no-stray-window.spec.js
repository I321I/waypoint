import assert from "node:assert/strict";

// 啟動斷言：WAYPOINT_E2E 自動開啟列表後，整個 app 只能存在一個 webview，
// 且必須是 list（hash 為 #view=list）。
//
// 為什麼需要這個測試：
// 之前回報「除了我開的筆記以外，有多餘的全黑視窗」——啟動階段若有任何
// 預先建立但未顯示 / 黑屏 / hidden 的 WebviewWindow，都會在這裡被抓到。
// （tauri-driver / WDIO 的 getWindowHandles() 會列出所有 webview，
// 不論 visible 與否，因此 hidden 的多餘視窗也會被檢出。）
describe("啟動時無多餘視窗", () => {
  before(async () => {
    // 等列表視窗載入完成（與其他 spec 一致，確認 SvelteKit 已掛載）
    await browser.waitUntil(
      async () => {
        try {
          return (await browser.getPageSource()).includes("WAYPOINT");
        } catch {
          return false;
        }
      },
      { timeout: 20_000, timeoutMsg: "列表視窗未出現 WAYPOINT 字串" },
    );
  });

  it("WAYPOINT_E2E 自動啟動後只能存在 list 視窗", async () => {
    const handles = await browser.getWindowHandles();
    const labels = [];
    for (const h of handles) {
      await browser.switchToWindow(h);
      const url = await browser.getUrl();
      labels.push(url);
    }
    const onlyList = labels.every((u) => u.includes("view=list"));
    assert.equal(
      labels.length,
      1,
      `啟動時 webview 視窗數應為 1，實際 ${labels.length}：${JSON.stringify(labels)}`,
    );
    assert.ok(
      onlyList,
      `啟動時所有視窗都應為 list（#view=list），實際：${JSON.stringify(labels)}`,
    );
  });
});
