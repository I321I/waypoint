# 2026-04-27 — 透明拉桿、穿透 hotkey、改名同步、視窗拖曳、儲存修復

## 背景

使用者在 v0.1.16 後回報 7 個議題，本 spec 統一處理：

1. 透明度設定無視覺效果
2. 透明拉桿想移到 titlebar 並選定樣式
3. 在 Edge focus 時按 `Ctrl+Shift+T` 不觸發穿透
4. 在筆記改完標題後，列表沒有立即更新
5. 在列表右鍵改名後，已開啟的筆記視窗未同步新標題
6. 設定／使用說明視窗點 titlebar 完全無法拖曳
7. 筆記內容在按 ✕ 或結束 app 後消失（hide 後再開仍在）

範圍：UI／前端互動修正 + 後端原子寫檔 + 啟動期 hotkey 註冊回饋。

## 議題 1 — 透明度套用層級錯誤

### 根因
`body.note-view` 已設 `background: rgba(30,30,30, var(--note-alpha))`，但內層 `.note-window` 仍使用不透明的 `var(--bg-primary)`，把 body 的 alpha 蓋掉。

### 設計
- 把 `--note-alpha` 套用到 `.note-window` 的 `background`（與 body 同 rgba 公式）。
- 同時保留 `body.note-view` 的 alpha 設定，避免 webview 切換時閃白。
- 不引入新機制；既有 SettingsPanel 拉桿的 `applyOpacity` 流程保留邏輯，僅 CSS 層調整。

## 議題 2 — 透明拉桿移至 titlebar

### 設計
採用 mockup **樣式 A：極簡橫條**（36px 寬）。

- 位置：Note titlebar 按鈕區，**穿透圓點左邊**。
- 視覺：4px 高軌道、10px 圓 thumb、accent 色。
- 範圍：`min=10`、`max=100`、`step=5`。
- 元件：抽出 `<TitlebarOpacitySlider>` Svelte 元件，props `{ value, onInput }`，避免邏輯散在 NoteWindow。
- 互動：`pointer-events: auto`，不被 `data-tauri-drag-region` 截走（在 mousedown handler 已 `target.closest("button, input")` early return）。
- 移除 SettingsPanel 中的「透明度」欄位；只剩字體大小（避免兩處同步）。
- 既存的 `applyOpacity` / `--note-alpha` CSS variable 機制不變。

## 議題 3 — 穿透快捷鍵被佔用無回饋

### 根因
Tauri `global_shortcut` 走 OS RegisterHotKey，先到先得。Edge 不是搶佔者；常見的搶佔來源是其他全域 hotkey 工具（剪貼簿管理員、PowerToys、IME 等）。當註冊失敗時，現況只寫進 log 檔，使用者看不到。

### 設計

**A. 啟動期回饋**
- `register_passthrough_hotkey` 失敗時，除既有 log 外，主動：
  1. 顯示 tray 通知氣泡：「穿透快捷鍵 `Ctrl+Alt+T` 註冊失敗，可能已被其他程式占用。請至設定更換。」
  2. 在 settings 視窗的「穿透模式快捷鍵」section 顯示紅色警示文字。
- 透過 `AppState` 新增 `passthrough_hotkey_registered: AtomicBool` 欄位，前端用既有 `configApi.get` 擴充欄位讀取。

**B. 預設值改為 `Ctrl+Alt+T`**
- 衝突軟體較少（Edge 預設不綁、PowerToys 預設不綁）。
- 既有使用者 config 已寫 `Ctrl+Shift+T` 不會被覆蓋；只改 `default_passthrough_hotkey()` 與文件預設範例。

**C. 設定衝突清單**
- 內建一份本地清單（不顯示給使用者）：
  - `Ctrl+Shift+T`（瀏覽器：reopen tab；常見衝突）
  - `Ctrl+Shift+N`（瀏覽器：new private window）
  - `Ctrl+Alt+Del`（系統保留）
  - `Win+L`（系統保留）
  - `Ctrl+Esc` / `Win+任意`（系統保留）
- 在 SettingsWindow 使用者按下擷取的快捷鍵後，**儲存前**檢查；若命中清單，顯示提示：「`Ctrl+Shift+T` 在 Edge / Chrome 為「重新開啟分頁」，可能無法在這些視窗觸發。仍要使用？」
- 提示框含「仍要使用」「換一組」兩按鈕，不阻擋使用者堅持設定。

## 議題 4 — 筆記改標題，列表沒立即更新

### 設計
- NoteWindow `scheduleSave` 寫檔成功後，比對前次標題；若 `parseTitleContent(body).title` 改變，emit `waypoint://note-title-changed`，payload `{ noteId, contextId, newTitle }`。
- ListWindow 在 mount 時 `listen("waypoint://note-title-changed")`，更新對應 `NoteItem` 的 `title` reactive 變數。
- 卸載時 `unlisten`。

## 議題 5 — 列表右鍵改名，已開筆記未更新

### 設計
- ListWindow 的 `renameCtx` / `submitRename` 成功後，emit `waypoint://note-renamed-from-list`，payload `{ noteId, contextId }`。
- NoteWindow `onMount` 時 `listen("waypoint://note-renamed-from-list")`，命中自己的 `noteId` 即重新 `notesApi.read` 並更新 `title` + `body` + editor 內容。
- 為避免雙向 echo（Note 改 → emit → List 收 → emit → Note 收），兩條 channel 名稱不同（`note-title-changed` 來自 NoteWindow；`note-renamed-from-list` 來自 ListWindow）。

## 議題 6 — 設定／使用說明視窗無法拖曳

### 根因（已驗證）
- NoteWindow 的拖曳能動，靠兩個關鍵：
  1. `.note-title` span 加了 `pointer-events: none`，讓 mousedown 直接打到 `.titlebar`。
  2. fallback 用同步 `windowsApi.startDragging(label).catch(() => {})`，沒有 `await getCurrentWindow()`。
- SettingsWindow / HelpWindow 缺少 (1)，且用 `await getCurrentWindow().startDragging()`，async 引入微 task 延遲時 mousedown event loop 早已結束。

### 設計
- 抽出 `<DraggableTitlebar>` Svelte slot 元件：
  - 接 `label: string` prop
  - render `<div class="titlebar" data-tauri-drag-region on:mousedown={...}>`
  - 內含 `<slot name="left" />` `<slot />` `<slot name="right" />`
  - mousedown handler 同步呼叫 `windowsApi.startDragging(label).catch(() => {})`，跳過 button/input/textarea/select/a
  - 內部任何 text span 自動套 `pointer-events: none`（透過 css class）
- SettingsWindow / HelpWindow / ListWindow / NoteWindow 改用此元件，移除各自的 inline mousedown handler。
- Playwright 渲染測試 `*.drag.render.test.pw.ts`：mock `windowsApi.startDragging`，模擬 mousedown 在 titlebar 中段，斷言 spy 被呼叫且 label 正確。每個視窗一案。

## 議題 7 — 筆記內容遺失

### 根因
1. **A4（app exit）**：`cmd_exit_app` 直接 `app.exit(0)`；webview 被殺前，前端 `flushPendingSave` 沒機會跑。
2. **A1/A2（按 ✕）**：`scheduleSave` 雖有 `flushPendingSave` + `await`，但 `std::fs::write` 不是 atomic；極端情況 IPC reply 已回但檔案尚未 fsync 即被殺，下次讀取看到空檔。

### 設計（採 L1 + L2 + L3 三層）

**L1 — Atomic save（後端）**
- `save_content` 改為「寫到 `content.md.tmp` → `fs::rename` 取代 `content.md`」。
- 同 pattern 套用 `save_settings`（json）。
- 加 cargo test：寫入後立即斷電模擬（mock fs 中斷在 rename 前，原檔須完好）。

**L2 — Save on input（前端）**
- 移除 `scheduleSave` 的 500ms debounce，改為 100ms（足以合併連續打字、又能在使用者準備關閉前完成大部分寫入）。
- `handleTitleInput` / `handleContentUpdate` 共用同一條 100ms 節流。
- Editor 的 `update` event 在 IME composition 結束時也應觸發；確認 TipTap 的 `onUpdate` 會在 composition end 觸發（已是預設行為，不需修改）。

**L3 — App exit flush（後端 ↔ 前端 round-trip）**
- 新增 `cmd_exit_app_with_flush(app)`：
  1. 後端對所有 `note-*` webview emit `waypoint://flush-and-save-now`。
  2. 等待 `flush-ack` 回傳，或 800ms timeout（whichever earlier）。
  3. 之後才 `app.exit(0)`。
- NoteWindow `onMount` listen `waypoint://flush-and-save-now`，呼叫 `flushPendingSave` 後 emit `waypoint://flush-ack` 帶 `noteId`。
- `cmd_restart_app` 使用同一機制（在 `snapshot_open_windows` 之前 flush）。
- tray 「結束 Waypoint」與 SettingsWindow 重啟流程改呼 `cmd_exit_app_with_flush` / `cmd_restart_app`（後者已存在，補 flush 步驟）。

**測試**
- cargo test：atomic save 中斷情境。
- Vitest：debounce 改 100ms 不影響正確性。
- Playwright：模擬「打字 → 立刻按 ✕」確認 `notesApi.saveContent` 被呼叫一次，內容正確。
- E2E（Windows 必跑）：開筆記 → 打字 → tray 結束 → 重新啟動 → 內容仍在。

## 影響範圍

| 檔案 | 變動 |
|------|------|
| `src/windows/NoteWindow.svelte` | 加入 titlebar 透明拉桿；改用 DraggableTitlebar；emit title-changed；listen renamed-from-list；listen flush-and-save-now；debounce 100ms |
| `src/windows/note/SettingsPanel.svelte` | 移除透明度欄位 |
| `src/windows/note/TitlebarOpacitySlider.svelte` | **新檔** |
| `src/windows/DraggableTitlebar.svelte` | **新檔** |
| `src/windows/SettingsWindow.svelte` | 改用 DraggableTitlebar；hotkey 衝突檢查；註冊失敗警示 |
| `src/windows/HelpWindow.svelte` | 改用 DraggableTitlebar |
| `src/windows/ListWindow.svelte` | 改用 DraggableTitlebar；listen note-title-changed；emit note-renamed-from-list |
| `src/lib/hotkeyConflicts.ts` | **新檔**：常見衝突清單 + 比對函式 |
| `src-tauri/src/storage/notes.rs` | save_content / save_settings 改 atomic |
| `src-tauri/src/storage/app_config.rs` | 預設 passthrough hotkey 改 `Ctrl+Alt+T` |
| `src-tauri/src/state.rs` | 新增 `passthrough_hotkey_registered` |
| `src-tauri/src/lib.rs` | register 失敗時 tray notification + 標 state |
| `src-tauri/src/commands/config_cmd.rs` | get_config 回傳 hotkey 註冊狀態 |
| `src-tauri/src/hotkey/mod.rs` | 新增 `cmd_exit_app_with_flush`；`cmd_restart_app` 加 flush 步驟 |
| `src-tauri/src/tray/mod.rs` | 「結束」改呼 `cmd_exit_app_with_flush` |

## 不做（YAGNI）
- 儲存失敗 toast（L4）— 採 atomic + flush 已能消除絕大多數丟資料；toast UI 暫不做。
- Hotkey 鍵盤 hook 替代 RegisterHotKey — 避開「可疑軟體」風險。
- SettingsPanel 透明欄位保留 — 保留會讓兩處同步出 bug。

## 驗收

1. NoteWindow titlebar 出現 36px 拉桿，拖動可改變透明度（即時生效，含內層 `.note-window`）。
2. SettingsPanel 不再有透明欄位。
3. SettingsWindow / HelpWindow titlebar 可拖曳（mousedown 任何文字區域皆可）。
4. 在 SettingsWindow 嘗試設定 `Ctrl+Shift+T` 會跳衝突提示。
5. 預設安裝後 passthrough hotkey 為 `Ctrl+Alt+T`，能在 Edge 觸發。
6. 在 NoteWindow 改標題 → ListWindow 立即顯示新標題（500ms 內）。
7. 在 ListWindow 右鍵改名 → 已開的 NoteWindow 立即顯示新標題與更新後 content。
8. 打字 → 立刻按 ✕：重開該筆記內容仍在。
9. 打字 → 系統列「結束 Waypoint」→ 啟動 → 內容仍在。
10. cargo test / npm test / npm run test:render / e2e-windows 全綠。
