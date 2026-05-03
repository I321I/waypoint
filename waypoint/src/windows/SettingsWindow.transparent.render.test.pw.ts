import { test, expect } from '@playwright/test';

test('settings window 透明文字 toggle 預設 ON 並可切換', async ({ page }) => {
  await page.addInitScript(() => {
    (window as any).__TAURI_INTERNALS__ = {
      metadata: {
        currentWindow: { label: 'settings' },
        currentWebview: { label: 'settings', windowLabel: 'settings' },
      },
      invoke: (cmd: string, _args?: any) => {
        if (cmd === 'get_app_config') return Promise.resolve({
          hotkey: 'Ctrl+Shift+Space',
          passthroughHotkey: 'Ctrl+Shift+Q',
          showInTaskbar: true,
          transparentIncludesText: true,
          passthroughHotkeyRegistered: true,
        });
        if (cmd === 'is_autostart_supported') return Promise.resolve(false);
        if (cmd === 'get_autostart') return Promise.resolve(false);
        if (cmd === 'is_passthrough_hotkey_registered') return Promise.resolve(true);
        if (cmd === 'get_transparent_includes_text') return Promise.resolve(true);
        if (cmd === 'set_transparent_includes_text') return Promise.resolve();
        if (cmd === 'plugin:event|emit') return Promise.resolve();
        return Promise.resolve(null);
      },
      transformCallback: () => 0,
      unregisterCallback: () => {},
      convertFileSrc: (s: string) => s,
    };
  });
  await page.goto('http://localhost:4173/#view=settings');
  await page.waitForLoadState('networkidle');
  const toggle = page.locator('input[data-testid="transparent-includes-text"]');
  await toggle.waitFor({ state: 'attached' });
  await expect(toggle).toBeChecked();
  // toggle off
  await toggle.click();
  await expect(toggle).not.toBeChecked();
});
