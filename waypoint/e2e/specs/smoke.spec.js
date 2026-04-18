import assert from "node:assert/strict";

describe("列表視窗 smoke", () => {
  before(async () => {
    // 等 SvelteKit 載入完成
    await browser.waitUntil(
      async () => {
        try {
          const src = await browser.getPageSource();
          return src.includes("WAYPOINT");
        } catch {
          return false;
        }
      },
      { timeout: 20_000, timeoutMsg: "列表視窗未出現 WAYPOINT 字串（頁面可能白屏或未載入）" },
    );
  });

  it("body 背景不是白色（防白屏）", async () => {
    const bg = await browser.execute(
      () => getComputedStyle(document.body).backgroundColor,
    );
    assert.notEqual(bg, "rgb(255, 255, 255)", "body 背景不應為白色");
    assert.notEqual(bg, "rgba(0, 0, 0, 0)", "body 背景不應為透明");
  });

  it("WAYPOINT 標題顯示於 titlebar", async () => {
    const el = await browser.$(".app-name");
    assert.ok(await el.isExisting(), ".app-name 元素不存在");
    const text = await el.getText();
    assert.equal(text, "WAYPOINT");
  });

  it("titlebar 四個按鈕齊全（?、⚙、⇊、✕）", async () => {
    const titles = ["使用說明", "設定", "收起全部", "關閉列表"];
    for (const t of titles) {
      const btn = await browser.$(`button[title="${t}"]`);
      assert.ok(await btn.isExisting(), `按鈕「${t}」不存在`);
      assert.ok(await btn.isDisplayed(), `按鈕「${t}」不可見`);
    }
  });

  it("titlebar 僅一個關閉按鈕（無雙層 X）", async () => {
    const btns = await browser.$$(".titlebar button");
    let xCount = 0;
    for (const b of btns) {
      const txt = (await b.getText()).trim();
      if (txt === "✕") xCount += 1;
    }
    assert.equal(xCount, 1, `列表 titlebar 應僅有 1 個 ✕，實際 ${xCount}`);
  });

  it("全域筆記 section 存在", async () => {
    const src = await browser.getPageSource();
    assert.ok(src.includes("全域筆記"), "找不到「全域筆記」section");
  });
});
