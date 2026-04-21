# 設計：穿透模式 + 視窗/列表/透明度修復批次

日期：2026-04-21
平台優先：Windows（WebView2）；Linux 同步維持綠燈

## 背景與目標

一次處理使用者回報的 10 項需求：8 個 bug/體驗修正 + 1 個新功能（穿透模式）+ 1 項回歸測試。

## 需求清單（最終定案）

| # | 項目 | 類型 |
|---|---|---|
| R1 | 啟動後除了使用者開的筆記外，不應出現多餘的全黑 Waypoint 視窗 | bug |
| R2 | 加入測試覆蓋 R1：除了已開的視窗外不會跳出多餘 Waypoint 視窗 | test |
| R3 | 筆記視窗永遠最上層（always-on-top） | feat |
| R4 | 移除筆記的專屬叫出/隱藏快捷鍵（只保留列表叫出 + 新的穿透快捷鍵） | feat |
| R5 | 透明度真正可看穿到下層視窗（目前只是把 bg 變白） | bug |
| R6 | 透明度滑桿 100% 時拇指不在最右側 | bug |
| R7 | 列表「右鍵 → 刪除」確認框在預設列表寬度下顯示不全 | bug |
| R8 | 移除列表的自動隱藏功能 | feat |
| R9 | 新增「穿透」（click-through）功能：全域快捷鍵切換、UI 指示、tray 開關 | feat |
| R10 | 移除筆記的最小化按鈕 [—] | feat |
| R11 | 筆記 titlebar 加「收起全部並儲存」按鈕（沿用既有 `cmd_collapse_all`） | feat |
| R12 | 使用說明 / 設定視窗目前無法拖曳，需修復 | bug |
| R13 | 任一 Waypoint 視窗（list/note/settings/help）可見時 → 工作列顯示大圖示應用程式；全部隱藏（純背景）→ 不顯示。可在列表設定開關此功能。tray 小圖示不動。 | feat |

## 細節設計

### R1 / R2 — 多餘的全黑視窗

**假設**：在現有的 setup 流程中（`src-tauri/src/lib.rs` 或 `src/main.rs`），可能有預先建立的 hidden 視窗（例如 list/help/settings 預先 create）導致看到一個無內容的黑色 frame。

**做法**：
- 釐清 setup 中所有 `WebviewWindowBuilder::new(...)` 呼叫，確認每個視窗只在被使用者操作時建立
- 任何「需要預掛載」的視窗，必須在建立前同時 set `visible(false)` + 確保 webview 載入完成才可顯示
- 加入 startup 後的「視窗清單」斷言：只允許 list 視窗存在（隱藏狀態），不應有任何 note/settings/help 視窗

**測試（R2）**：
- 新增 E2E spec `e2e/specs/no-stray-window.spec.js`：啟動 app → 透過 tauri-driver 取得目前 window 數量與 label → 斷言只有預期的 system 視窗（list 是唯一在啟動後存在的）
- Vitest 補充：純函式層次的 setup 計數（如有）

### R3 — 筆記永遠最上層

- 在筆記視窗建立時 `.always_on_top(true)`；既有設定不調
- 不提供「取消最上層」UI（不在本 spec 範圍）

### R4 — 移除筆記專屬快捷鍵

- 找到目前註冊的「叫出/隱藏筆記」global shortcut（`src-tauri/src/hotkey/`）並移除註冊與設定 UI
- 列表的叫出快捷鍵保留
- 不影響 R9 新增的穿透快捷鍵

### R5 — 真透明

- Tauri 視窗設定：筆記視窗 builder 加 `.transparent(true)` + `.decorations(false)`（已是無 decoration）
- CSS：`--bg-primary` 改成 `rgba(背景色, α)`；α 由現有透明度設定 store 控制
- 移除「白色 over-paint」邏輯（如有）
- 範圍：α ∈ [0.05, 1.0]，0.05 仍保留邊框與 dot 可見

### R6 — 透明度滑桿 100% 不在最右

- 排查 `SettingsPanel.svelte`（`src/windows/note/`）的滑桿 markup：很可能是 `<input type="range">` thumb 寬度 + track padding 沒對齊，或 `max` 設成非 100 的值
- 修正方式：`min=0 max=100 step=1`，CSS `appearance: none` + 自訂 thumb，並讓 track 與 thumb 中心點對齊容器邊界
- 加 Vitest：把 value 設為 100 → 計算 thumb 的 `left%` = 100

### R7 — 刪除確認框被截斷

- 列表預設寬度（檢查 `src/windows/list/` 或 ListWindow 內 default size）下，現行 confirm dialog 是 inline 還是 native？
- 改成：confirm dialog 不受列表寬度限制 — 用獨立的 Tauri dialog（`@tauri-apps/plugin-dialog`）或 portal 浮層脫離 list 容器
- 內容必須容納「刪除「<note title>」？」的長標題；按鈕至少 [取消][刪除] 並排無折行
- E2E：在預設列表大小下執行右鍵→刪除→截圖斷言「刪除」按鈕在可視區內

### R8 — 移除列表自動隱藏

- 移除 list 的 autohide setting + 相關 timer / focus 偵測邏輯
- 列表只會被「叫出快捷鍵」或「結束 Waypoint」開關
- 移除設定面板對應 UI 與儲存欄位（保留 migration：舊 config 內的欄位讀進來忽略即可）

### R9 — 穿透功能（核心新功能）

#### 行為

- **快捷鍵**：預設 `Ctrl+Shift+T`，可在列表設定面板修改（沿用既有「叫出列表快捷鍵」的同一套設定 UI 機制）
- **作用域**：全域同步 — 按下後**所有筆記**同時切換到當前狀態的反面
  - 規則：若目前「任一」筆記為非穿透 → 全部設為穿透；若全部已穿透 → 全部設為非穿透
- **持久化**：每個筆記**個別**記住自己的最後穿透狀態（同筆記其他設定一樣 per-note 儲存到 note settings）
  - App 重啟後，每個被開啟的筆記回復自己上次的穿透狀態
  - 全域快捷鍵會 override 個別狀態，但 toggle 後的新狀態會即時寫回各 note settings
- **dot 點擊**：行為等同全域快捷鍵（按任一筆記的 dot = 全部同步切換）
- **tray 右鍵選單**：新增「穿透：開 / 關」項，行為等同快捷鍵；菜單文字反映目前狀態

#### Note titlebar 新版面

```
[標題……] [● dot] [⇊ 收起全部並儲存] [▢ 最大化/還原] [✕ 儲存並關閉]
```

- 移除既有 [—] 最小化（R10）
- 新增 dot：直徑 8px，右上角間距同其他按鈕；綠 = `#5cb85c`、黃 = `#ffb454`（黃帶 6px 光暈 box-shadow）
- 新增「⇊ 收起全部並儲存」按鈕：icon 與列表的 collapse-all 按鈕一致；行為見 R11
- dot 在「⇊」按鈕的左邊，與其他按鈕同列（titlebar buttons row）

#### 穿透開啟時的視窗行為

- Windows: `set_ignore_cursor_events(true)`（Tauri API）
- 滑鼠事件全部穿透到下層視窗，**包括 dot 自己**
- 解除穿透只能透過：全域快捷鍵 / tray 右鍵
- 視覺上 dot 變黃 + 帶光暈，邊框與內容維持原樣（採用使用者選的 B 風格而非 D；簡潔不干擾）

#### Tray 指示

- Tray icon 本體**不變**（避免 icon 抖動）
- 右鍵選單第一項顯示：`● 穿透：開` 或 `○ 穿透：關`，狀態切換時動態更新 menu item 文字 + 前綴符號

### R10 — 移除筆記最小化

- 純 markup 移除：`NoteWindow.svelte` 的 `<button on:click={handleMinimize}>—</button>` 連同 handler 一併刪除
- E2E smoke spec 同步把「最小化按鈕存在」斷言移除

### R11 — 收起全部並儲存按鈕

- 在 R9 的 titlebar 改造中一起加入
- icon：`⇊`（與列表的 collapse-all 按鈕一致），title 屬性「收起全部並儲存」
- 行為：emit `waypoint://collapse-all-requested` 事件 — **不直接呼叫 `cmd_collapse_all`**
- 既有的 `ListWindow.svelte` 已監聽此事件，會先 `sessionApi.save(currentContextId, { openContextNotes, openGlobalNotes })` 再呼叫 `windowsApi.collapseAll()`，效果與使用者按列表的 `⇊` 完全相同
- **前提**：list 視窗必須在 app 啟動後即刻 create（即使 hidden）以保持 svelte 元件 + listener 存活；這在 R1 重新檢視多餘視窗時一併確認（list 是允許預掛載的唯一視窗）
- 後備：Rust 端 hotkey handler 已有 200ms delay safety fallback（list 不在就直接 collapse），筆記事件路徑同樣受惠
- E2E：開兩個筆記 → 在筆記點 `⇊` → 視窗全關 → 重啟 app → 兩個筆記都還原

### R12 — Help / Settings 視窗拖曳

- 現況：`HelpWindow.svelte` (line 6) 與 `SettingsWindow.svelte` (line 93) 的 titlebar 只有 `data-tauri-drag-region`，缺少 NoteWindow 已有的 `handleTitlebarMousedown` → `windowsApi.startDragging(...)` fallback（NoteWindow 是因為這個原因才加的，見既有註解：「data-tauri-drag-region 在某些平台偶爾失效」）
- 修法：在兩個 svelte 加上同樣的 `on:mousedown={handleTitlebarMousedown}` handler，傳入對應 window label（`help`、`settings`）
- 跳過按鈕/輸入框：handler 內 `if (target.closest("button") || target.closest("input")) return`（與 NoteWindow 一致）
- E2E：在 Help / Settings 視窗對 titlebar 空白處 mousedown + drag → 視窗位置改變

### R13 — 工作列大圖示動態顯示

- **目標**：當 Waypoint 有任一可見視窗時，作業系統工作列上要看得到 Waypoint 應用程式（含 icon + 視窗清單，可 alt-tab、可從工作列右鍵切換各視窗）。當所有 Waypoint 視窗都隱藏（純背景 + tray）時，工作列不顯示大圖示。tray 小圖示行為**完全不變**。
- **實作**：用 Tauri 的 `WebviewWindow::set_skip_taskbar(bool)`
  - 可見視窗數 > 0 且設定開啟 → 對所有可見視窗 `set_skip_taskbar(false)`
  - 可見視窗數 = 0 或設定關閉 → 全部 `set_skip_taskbar(true)`
- **觸發點**：每次視窗 show / hide / create / close 時重算一次（Rust 端集中函式 `refresh_taskbar_visibility()`）
  - hook 進現有 `open_list_window` / `open_note_window` / collapse_all / handleClose 等路徑
- **設定**：在列表設定面板新增 `showInTaskbar: boolean`（預設 `true`），存到 `app_config`；`refresh_taskbar_visibility` 讀此 flag 決定是否啟用此特性
- **跨平台**：`set_skip_taskbar` 在 Windows / Linux 行為一致；macOS 用 dock，行為由 Tauri 抽象（次要平台，不在本次優先驗證範圍）
- **不影響**：tray icon、tray 右鍵選單、視窗 always_on_top 等屬性
- **測試**：
  - cargo test：`refresh_taskbar_visibility` 對給定 (visible_count, setting) → 預期 skip_taskbar 值
  - E2E：開列表 → 工作列出現 → 收起全部 → 工作列消失；關閉設定 flag 後不論視窗都不顯示

## 元件邊界

- **`src-tauri/src/hotkey/`**：新增穿透快捷鍵註冊與 handler；移除舊的筆記專屬快捷鍵
- **`src-tauri/src/commands/`**：新增 `cmd_set_passthrough(note_id, on)` 與 `cmd_toggle_passthrough_global()`；後者讀目前狀態 → 決定目標 → 對所有筆記呼叫前者並寫回 settings
- **`src-tauri/src/tray/`**：新增穿透 menu item，與後端狀態雙向綁定
- **`src/lib/types.ts`**：`NoteSettings` 加 `passthrough: boolean`（預設 false）
- **`src/lib/api.ts`**：加 `setPassthrough` / `togglePassthroughGlobal`
- **`src/windows/NoteWindow.svelte`**：titlebar 改造 + dot 元件
- **`src/windows/list/SettingsPanel`（或同等位置）**：穿透快捷鍵設定欄位
- **`src/windows/list/ConfirmDialog` 或新元件**：脫離列表寬度限制的 confirm

## 測試矩陣

| 需求 | Vitest | Playwright render | WDIO E2E (Windows) |
|---|---|---|---|
| R1/R2 | — | — | ✅ no-stray-window |
| R3 | — | — | ✅ note window has alwaysOnTop attr |
| R4 | ✅ hotkey 註冊清單不含舊 key | — | — |
| R5 | ✅ store 層的 alpha 範圍 | ✅ render 用 rgba bg | — |
| R6 | ✅ 100 → thumb left=100% | ✅ slider 100 截圖 | — |
| R7 | — | ✅ confirm dialog 完整可視 | ✅ 右鍵刪除流程 |
| R8 | ✅ autohide setting 已移除 | — | — |
| R9 | ✅ toggle 邏輯（混合狀態 → 全開 / 全開 → 全關 / 全關 → 全開） | ✅ dot 綠/黃渲染 | ✅ 快捷鍵 + tray 切換 + 視窗 ignoreCursorEvents 行為 |
| R10 | — | ✅ titlebar 無 [—] | ✅ smoke spec 更新 |
| R11 | — | ✅ titlebar 有 ⇊ | ✅ 點擊 → 所有筆記關閉 + session 已存 + 重啟還原 |
| R12 | — | — | ✅ Help / Settings titlebar 拖曳改變視窗位置 |
| R13 | ✅ refresh_taskbar_visibility 邏輯表 | — | ✅ 開列表 → 工作列有；收起 → 工作列無；關 flag → 永遠無 |

## 風險與已知盲點

- **R5 + R9 互動**：透明 + ignore_cursor_events 在 Windows WebView2 上是否衝突，需在 spike 階段確認；若衝突，dot 顏色仍須在透明背景下可辨識
- **R5 白屏盲點**：CLAUDE.md 已警告 `app.html` inline body style 可能遮蔽白屏 → 改 RGBA 時要同步調整 inline style，並在 render test 斷言 `getComputedStyle(body).background` 為 rgba 而非純色
- **R7 confirm 浮層**：若用獨立 Tauri 視窗，注意 always_on_top 與 list 同層級
- **R9 持久化 + 全域 toggle 的語意衝突**：已在「行為」段定義「全域 toggle 後即時寫回各 note settings」消除歧義

## 不在此範圍

- 透明度 + Acrylic 毛玻璃（之後另案）
- 穿透的「半穿透」（部分元素接收事件）
- 跨 context 的批次穿透切換（目前只有「全部」一檔）

## 落地順序（提供給後續 writing-plans 參考）

1. R1/R2（先封住多餘視窗，避免後續 E2E 噪訊）
2. R10、R11、R8、R12（純移除/新增 UI / 拖曳 fallback，不涉及複雜邏輯）
3. R3、R4、R6（小修）
4. R5（真透明，獨立 spike）
5. R9（穿透完整功能，依賴 R5 的視窗設定）
6. R7（confirm 改造）
7. 全測試矩陣綠燈 → merge dev/main → push master → 等 e2e-windows 綠 → tag
