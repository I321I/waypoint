import { test, expect } from "./fixtures";

test("列表視窗不白屏、顯示 WAYPOINT 標題", async ({ app }) => {
  const list = await app.findPage(/#view=list/);

  const bg = await list.evaluate(() =>
    window.getComputedStyle(document.body).backgroundColor
  );
  expect(bg).not.toBe("rgb(255, 255, 255)");
  expect(bg).not.toBe("rgba(0, 0, 0, 0)");

  await expect(list.locator("text=WAYPOINT")).toBeVisible({ timeout: 10_000 });
});
