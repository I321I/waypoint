import { defineConfig } from "@playwright/test";

// E2E 設定：驅動真實 Tauri binary（WebView2 on Windows）。
// 與 playwright.config.ts（render test, preview server）獨立。
export default defineConfig({
  testDir: "./e2e",
  testMatch: /.*\.spec\.ts$/,
  fullyParallel: false,
  workers: 1,
  timeout: 60_000,
  expect: { timeout: 10_000 },
  reporter: [["list"], ["html", { open: "never" }]],
  use: {
    trace: "retain-on-failure",
  },
});
