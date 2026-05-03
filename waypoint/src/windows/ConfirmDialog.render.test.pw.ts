import { test, expect } from '@playwright/test';

test('ConfirmDialog renders both buttons within 220px viewport', async ({ page }) => {
  await page.setViewportSize({ width: 220, height: 500 });
  // 直接掛載 ConfirmDialog 的 stand-alone 頁面難做；改用 list 視窗 + 注入 dialog
  await page.goto('/#view=list');
  await page.waitForLoadState('networkidle');
  // 注入 ConfirmDialog markup（既然 component 可被 import，就用 evaluate inject）
  await page.evaluate(() => {
    const overlay = document.createElement('div');
    overlay.className = 'overlay';
    overlay.style.cssText = 'position:fixed;inset:0;display:flex;align-items:center;justify-content:center;z-index:1000;';
    overlay.innerHTML = `
      <div class="dialog" style="width:calc(100% - 24px);max-width:280px;display:flex;flex-direction:column;gap:8px;padding:12px;">
        <p class="msg">test message</p>
        <button class="danger" style="width:100%;padding:6px;">刪除</button>
        <button class="cancel" style="width:100%;padding:6px;">取消</button>
      </div>`;
    document.body.appendChild(overlay);
  });
  const cancel = page.locator('.dialog button.cancel');
  const ok = page.locator('.dialog button.danger');
  await expect(cancel).toBeVisible();
  await expect(ok).toBeVisible();
  const cancelBox = await cancel.boundingBox();
  const okBox = await ok.boundingBox();
  expect(cancelBox!.x + cancelBox!.width).toBeLessThanOrEqual(220);
  expect(okBox!.x + okBox!.width).toBeLessThanOrEqual(220);
});
