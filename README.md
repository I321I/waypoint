# Waypoint

跨平台浮動筆記。一個快捷鍵叫出筆記列表，按應用程式自動切換不同的筆記集合，邊看筆記邊操作其他軟體。

[![Latest release](https://img.shields.io/github/v/release/I321I/waypoint)](https://github.com/I321I/waypoint/releases/latest)

![Waypoint](docs/screenshot.png)

## 特色

- **全域快捷鍵**（預設 `Ctrl+Shift+Space`）一鍵叫出列表 / 收起全部
- **Context 切換**：依當前前景應用程式（process 名或視窗標題）自動切換到該軟體的筆記集合，不同遊戲 / 工作的筆記分開不混
- **滑鼠穿透模式**（預設 `Ctrl+Shift+Q`）讓筆記變半透明且滑鼠/鍵盤直接穿過去操作下方應用程式 — 邊打 Boss 邊看攻略最舒服
- **筆記永遠最上層**、可獨立調整透明度（10–100% 滑桿在 titlebar）
- **Markdown 編輯**：所見即所得（# 自動變 H1、`**bold**` 變粗體、check list、code block、表格）
- **Session 還原**：上次開著的筆記、視窗位置、大小都會被記住
- **跨平台**：Windows、macOS、Linux（x86_64 / aarch64，含 Steam Deck）
- **資料純檔案**：所有筆記與設定存在 `~/waypoint/`，整個資料夾複製到別台電腦就能無縫接續

## 安裝

從 [Releases](https://github.com/I321I/waypoint/releases/latest) 下載對應平台檔案：

| 平台 | 檔案 |
|---|---|
| Windows x64 | `Waypoint_*_x64-setup.exe` 或 `_x64_en-US.msi` |
| macOS Universal | `Waypoint_*_universal.dmg` |
| Linux x86_64 | `_amd64.AppImage` / `.deb` / `.rpm` / `_x86_64.flatpak` |
| Linux aarch64 | `_aarch64.AppImage` / `.deb` / `.rpm` / `_aarch64.flatpak` |
| Steam Deck (推薦) | `_x86_64.flatpak`（Desktop Mode：`flatpak install --user ./io.github.i321i.waypoint_x86_64.flatpak`） |

## 基本操作

1. 安裝後啟動，按 `Ctrl+Shift+Space` 叫出列表視窗
2. 點 `+` 新增筆記（區域筆記 = 跟著當前 app；全域筆記 = 任何 app 都顯示）
3. 點筆記開啟視窗，輸入 markdown 即時 render
4. 視窗 titlebar 上的滑桿調透明度；按 `Ctrl+Shift+Q` 全部一起切換滑鼠穿透
5. 右鍵筆記可重新命名 / 複製到其他 context / 移動 / 刪除
6. `⇊` 收起全部（保留 session，下次再叫出）；`✕` 關閉單一筆記（永久關閉）

詳細說明見列表視窗右上角 `?` 按鈕。

## 開發

```bash
cd waypoint
npm install

# 開發模式
npm run tauri dev

# 前端單元測試
npm test

# 渲染測試（Playwright）
npm run build && npm run test:render

# Rust 單元測試
cd src-tauri && cargo test
```

E2E（WebdriverIO + tauri-driver）：

```bash
# Linux（WebKitGTK，可代理 Steam Deck）— 本機 act
export DOCKER_HOST=tcp://host.docker.internal:2375
act -j e2e-linux

# Windows（WebView2）— GitHub runner，push master 自動跑
```

開發規範與 CI 流程詳見 [CLAUDE.md](CLAUDE.md)。

## 授權

MIT
