import assert from "node:assert/strict";

describe("Waypoint smoke", () => {
  it("列表視窗不白屏、顯示 WAYPOINT 標題", async () => {
    // tauri-driver 啟動 binary 並自動 attach 第一個 WebView2 視窗（列表視窗，由 WAYPOINT_E2E 觸發）
    await browser.waitUntil(
      async () => {
        try {
          const src = await browser.getPageSource();
          return src.includes("WAYPOINT");
        } catch {
          return false;
        }
      },
      { timeout: 20_000, timeoutMsg: "列表視窗未出現 WAYPOINT 字串" },
    );

    const bg = await browser.execute(
      () => getComputedStyle(document.body).backgroundColor,
    );
    assert.notEqual(bg, "rgb(255, 255, 255)", "body 背景不應為白色");
    assert.notEqual(bg, "rgba(0, 0, 0, 0)", "body 背景不應為透明");
  });
});
