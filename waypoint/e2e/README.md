# E2E (tauri-driver + WebdriverIO)

使用 Tauri 官方 E2E 方案：`tauri-driver` 代理 `msedgedriver`（Windows WebView2）
或 `WebKitWebDriver`（Linux），透過 WebDriver 協議驅動 Tauri binary。

## 本機執行（Windows）

```powershell
cd waypoint
npm install
cargo install tauri-driver --locked
npm run build
cd src-tauri
cargo build --release
cd ..
$env:WAYPOINT_BINARY = "$PWD\src-tauri\target\release\waypoint.exe"
npm run test:e2e
```

## CI

`.github/workflows/e2e-windows.yml` 會安裝 tauri-driver 並跑 smoke。

## 為何不是 Playwright CDP

`--remote-debugging-port` 會破壞 Tauri 的 `http://tauri.localhost` asset scheme，
WebView2 導頁失敗。tauri-driver 走 WebDriver 協議，無此問題。
