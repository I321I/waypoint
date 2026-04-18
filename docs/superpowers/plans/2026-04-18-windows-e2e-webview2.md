# Windows E2E (WebView2) 實作計畫

**Goal:** 在 GitHub Actions windows-latest runner 上跑真實 WebView2 E2E smoke test，驗證列表視窗不白屏、有關鍵 UI；每次 PR/push 必跑，綠燈才能發布。

**Architecture:** Tauri release build → 啟動 binary 時帶 `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222` 環境變數 → Playwright 用 `chromium.connectOverCDP` 接上 → 透過多 page 對應 Tauri 多視窗 → 斷言 DOM/CSS。

**Tech Stack:** Tauri 2, Playwright (既有), Node 20, Windows WebView2 Runtime。

後續 Phase 2-5 另寫計畫，本文件只涵蓋 Phase 1。

---

## File Structure

```
waypoint/
├── e2e/
│   ├── fixtures.ts              # (new) launch binary + CDP connect helper
│   ├── smoke.spec.ts            # (new) Phase 1 smoke test
│   └── README.md                # (new) 說明如何本機跑 E2E
├── playwright.e2e.config.ts     # (new) E2E 專用設定，獨立於 playwright.config.ts
└── package.json                 # (modify) 加 test:e2e script

.github/workflows/
└── e2e-windows.yml              # (new) windows-latest E2E job
```

---

## Task 1: e2e/fixtures.ts

**Files:** Create `waypoint/e2e/fixtures.ts`

負責：從 env `WAYPOINT_BINARY` 讀 binary 路徑，spawn 時注入 `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`，retry 連 CDP（WebView2 啟動需 1-3 秒），expose Playwright fixture `app` 提供 `browser`、`getWindow(hashPattern)`。

關鍵實作：
- `chromium.connectOverCDP` 拿到 browser context
- `context.pages()` 可能有多個，用 `page.url()` 的 hash 過濾（`#/list`、`#/settings`、`#/note/xxx`）
- teardown 時 kill spawned process

## Task 2: playwright.e2e.config.ts

獨立 config。不用 webServer（binary 由 fixture 啟動）。testDir `./e2e`，單 worker（Tauri singleton），只跑 chromium project（CDP 模式 browserName 必須是 chromium）。

## Task 3: e2e/smoke.spec.ts

三個斷言：
1. 列表視窗存在（找到 `#/list` 的 page）
2. body background 不是 `rgb(255,255,255)`（白屏防線）
3. DOM 中找得到 "Waypoint" 字串或列表容器

## Task 4: package.json

加 `"test:e2e": "playwright test -c playwright.e2e.config.ts"`。

## Task 5: .github/workflows/e2e-windows.yml

Job 流程：checkout → setup node 20 → install rust stable → npm install → npm run build → `cargo build --release`（不用 tauri-action，直接拿 binary）→ `npx playwright install chromium`（其實不需要瀏覽器本體，只用 CDP client，可省略）→ `WAYPOINT_BINARY=...\\waypoint.exe npm run test:e2e`。

觸發：`on: [push, pull_request]`，只跑 master/PR。

## Task 6: push 驗證

commit 所有新檔案，push，監看 Actions。預期首次會失敗，常見問題：
- CDP 連不上 → 延長 retry timeout
- binary 路徑錯 → 調整 WAYPOINT_BINARY env
- 無頭 session WebView2 拒絕啟動 → 加 `--headless=new` 或確認 runner 有 desktop session

---

## Self-Review Notes

- 沒有覆蓋 controller、Gamescope、多螢幕等 Steam Deck 特有點（Phase 3 處理）
- Phase 1 只有 1 個 smoke test，不測跨視窗互動（Phase 4 處理）
- WebView2 CDP 在 Tauri 2 的可靠度未實測，若不 work 需 fallback 到 tauri-driver + WebDriverIO
