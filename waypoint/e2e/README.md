# E2E (WebView2) 測試

驅動真實 Tauri binary，透過 WebView2 CDP 遠端除錯埠接上 Playwright。

## 本機執行（Windows）

```powershell
cd waypoint
npm install
npm run build
cd src-tauri
cargo build --release
cd ..
$env:WAYPOINT_BINARY = "$PWD\src-tauri\target\release\waypoint.exe"
npm run test:e2e
```

## CI

`.github/workflows/e2e-windows.yml` 在每次 push / PR 觸發。
