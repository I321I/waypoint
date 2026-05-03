# 2026-05-03 — 筆記功能修正與設定擴充 Design

## 背景

使用者回報 Waypoint 0.1.21 有以下五個問題與需求，本 spec 一次處理：

1. 開著筆記時從列表刪除 → 筆記視窗沒被關閉、X 也關不掉；session 重開仍會復活已刪除筆記，第二輪重開後甚至以「透明殘殼」狀態出現
2. 透明功能目前是整個視窗（含文字）一起透明，需要在列表設定提供開關，讓使用者選擇透明時文字是否一起跟著透明
3. 筆記底部藍色 statusbar（區域 / Markdown）要拿掉；區域改寫到 titlebar 標題旁，例如 `1122-Global`、`我是誰-edge`；Markdown 提示資訊改放 README / 使用說明
4. Alt+F4 關筆記後，下次列表 toggle 時該筆記又被自動拉起來（X 鈕沒這問題）— Alt+F4 應與 X 行為一致
5. 筆記內設定面板要新增「刪除此筆記」功能，且刪除後列表必須同步更新

## 設計總覽

五個議題技術上彼此偶有交集（特別是 #1 與 #5 共用刪除事件、#4 與 #1 共用「強制關閉」路徑），但功能獨立，可拆成五組獨立 commit。

### 議題 1：刪除筆記的正確生命週期

**根因：**
- `notesApi.delete` 刪了檔案但沒廣播事件 → 開著該筆記的 NoteWindow 視窗繼續活著
- 使用者按 X 想關 → `handleClose` 走 `flushPendingSave` → 試圖寫一個已不存在的 note → 阻塞或錯誤，看起來「關不掉」
- session 寫入時不檢查 note 檔是否存在 → 把幽靈條目寫入
- session 還原時也不檢查 → 把不存在的 note 視窗 spawn 出來，因為讀不到資料只剩透明殼

**修正：**

1. Rust 端 `cmd_delete_note` 成功後 emit `waypoint://note-deleted { noteId, contextId }` 廣播到所有視窗
2. NoteWindow 監聽 `waypoint://note-deleted`，若 noteId 與 contextId 都符合自己 → 走「強制關閉」分支：
   - **跳過** `flushPendingSave`、`saveContent`、`saveSettings`
   - emit `note-closed`（讓 session 把自己從「打開清單」移除）
   - close window
3. `app_session.rs` 寫入「目前打開的筆記清單」前，過濾不存在的 note 檔
4. session 還原時 spawn 視窗前再做一次 note 存在檢查；不存在則略過並從 session 中清除

### 議題 2：透明設定 — 文字是否一起透明

**設定位置：** 列表視窗的設定面板（SettingsWindow.svelte）新增 toggle「透明時文字也透明」，**預設 ON**（保持目前行為）。屬於全域設定，所有筆記共用。

**儲存：** `AppConfig` 增加欄位 `transparent_includes_text: bool`（預設 `true`）。

**渲染：**
- 目前 `.note-window` 用 `background: rgb(30,30,30)` + inline `opacity: {α}` → CSS `opacity` 會穿透到所有子元素
- 改用 CSS 變數 + `rgba()` 背景：`.note-window { background-color: rgba(30,30,30,var(--note-bg-alpha)) }`，`--note-bg-alpha` 由 inline style 設定為 `note.settings.opacity`
- 當 `transparent_includes_text === true`：再加上 `opacity: var(--note-bg-alpha)` → 等同現況（文字也跟著透明）
- 當 `false`：只 rgba 背景透明，子元素 `opacity: 1`

**避免重蹈覆轍的驗證計畫：**
- 透明的物理鏈路（已驗證可用）：`WebviewWindowBuilder::transparent(true)` + `decorations(false)` + `body.note-view { background: transparent !important }`，三者缺一不可，本次修改不能動到這三點
- 內部 chrome 背景審計：盤點 NoteWindow.svelte 內所有 `background:` 規則（titlebar、editor area、settings panel、toolbar、title-input），凡是 opaque rgb 一律改 `transparent` 或 rgba，否則「文字不透明、背景透明」時，子元件的 opaque 矩形會擋住桌面
- tiptap ProseMirror 預設背景若是 opaque，需 override 成 transparent

**事件廣播：** 列表設定 toggle 改變 → emit `waypoint://config-changed`，所有 NoteWindow 監聽並切換 class。

### 議題 3：移除底部藍 bar、區域標籤改到 titlebar

**移除：** `NoteWindow.svelte:200-203` 的 `.statusbar` 區塊（含「區域 / Markdown」字樣）與相關 CSS。

**Titlebar 標題格式：**
- `{title || "Untitled"}-{contextId ?? "Global"}`
- 全域：`1122-Global`
- 區域：`我是誰-edge`、`筆記-code` 等
- 接受筆記名本身含 `-` 的情況（例如 `2026-05-03 日記-Global`）

**README / 使用說明同步：** 在 README 與 HelpWindow 內容註明「筆記內容支援 Markdown 語法」。

### 議題 4：Alt+F4 行為與 X 一致

**根因：**
- X 鈕呼叫 `handleClose`：`flushPendingSave` → emit `note-closed` → close window。`note-closed` 會讓 session 把該筆記從「目前打開清單」移除 → 下次列表 toggle 不會自動恢復
- Alt+F4 走 OS / Tauri 預設關窗路徑，**不經過** `handleClose`，因此 session 仍視為「打開中」 → 下次列表 toggle 又把它拉起來

**修正：** NoteWindow.svelte 在 onMount 註冊：

```js
import { getCurrentWindow } from '@tauri-apps/api/window';
const unlisten = await getCurrentWindow().onCloseRequested(async (e) => {
  e.preventDefault();
  await handleClose();
});
```

這樣 Alt+F4、taskkill、其他 OS 關窗訊號全部走 X 的同一條路徑。

**列表 toggle 恢復邏輯不動：** 維持目前的「toggle 開列表時，恢復 session 中標記為打開的筆記」。X / Alt+F4 既然都會把自己從打開清單移除，這個邏輯本身就會做出「正確」行為。

### 議題 5：筆記內刪除此筆記

**UI：** `SettingsPanel.svelte`（筆記內設定面板）底部加紅色按鈕「刪除此筆記」。

**流程：**
1. 點按鈕 → 彈 ConfirmDialog
2. 確認 → 呼叫 `notesApi.delete(contextId, noteId)`
3. Rust 走議題 1 同一條刪除路徑（刪檔 + emit `waypoint://note-deleted`）
4. 自己視窗收到事件 → 強制關閉
5. 列表視窗收到事件 → 從清單移除該 note

**ConfirmDialog 重用：** 將 `waypoint/src/windows/list/ConfirmDialog.svelte` 移到 `waypoint/src/windows/ConfirmDialog.svelte`，列表與筆記兩端共用。

## 測試計畫

### 單元測試（Vitest / Rust）

- Rust：`cmd_delete_note` 成功後會 emit `waypoint://note-deleted`
- Rust：`app_session` 寫入時過濾不存在的 note 檔
- Rust：`AppConfig` 新欄位 `transparent_includes_text` 預設 `true`、序列化往返一致
- Vitest：NoteWindow 收到 `note-deleted` 事件且 noteId/contextId 符合 → 觸發強制關閉路徑（不呼叫 saveContent）
- Vitest：titlebar 標題格式 `{title}-{contextId ?? "Global"}`
- Vitest：SettingsPanel 點刪除按鈕 → 彈 ConfirmDialog → 確認後呼叫 `notesApi.delete`

### Playwright Render Tests

- `.statusbar` 已從 NoteWindow 移除（DOM 中找不到）
- titlebar 標題：全域筆記顯示 `xxx-Global`、區域筆記顯示 `xxx-{contextId}`
- 透明文字選項 OFF + α=0.3 時：`.note-window` computed `opacity === 1` 且 `background-color` 含 alpha < 1；`.note-title` computed `opacity === 1`
- 透明文字選項 ON + α=0.3 時：`.note-window` computed `opacity ≈ 0.3`（與目前行為一致）
- SettingsPanel 出現「刪除此筆記」按鈕；點擊 → ConfirmDialog 顯示

### E2E（WebdriverIO，Windows + Linux）

- 開列表 + 開筆記 → 在列表刪除該筆記 → 筆記視窗自動關閉
- 開筆記 → Alt+F4 關閉 → 觸發列表 toggle → 該筆記不被自動拉起
- 開列表 + 開筆記 → 在筆記內設定面板按刪除 → 確認 → 列表同步移除該 note，筆記視窗自動關閉
- 刪除筆記後完整 quit + relaunch → 不會出現透明殘殼視窗

## 範圍外（Out of scope）

- 筆記排序、筆記匯出、批次刪除
- 透明度滑桿位置或樣式調整
- contextId 的偵測 / normalize 邏輯（沿用現狀）
- ConfirmDialog 樣式重設計

## 影響檔案清單（預估）

- `waypoint/src/windows/NoteWindow.svelte` — 議題 1, 3, 4, 全部要動
- `waypoint/src/windows/note/SettingsPanel.svelte` — 議題 5
- `waypoint/src/windows/SettingsWindow.svelte` — 議題 2
- `waypoint/src/windows/ListWindow.svelte` — 議題 1（監聽 note-deleted）
- `waypoint/src/windows/ConfirmDialog.svelte`（新位置） + `list/ConfirmDialog.svelte`（移除） — 議題 5
- `waypoint/src/lib/api.ts`、`stores.ts`、`types.ts` — 配置欄位、事件 typing
- `waypoint/src-tauri/src/commands/notes.rs` — emit note-deleted
- `waypoint/src-tauri/src/storage/app_config.rs` — 新欄位
- `waypoint/src-tauri/src/storage/app_session.rs` — 過濾不存在的 note
- `waypoint/src-tauri/src/storage/notes.rs` — 刪除流程
- `README.md`、`waypoint/src/windows/HelpWindow.svelte` — Markdown 說明同步
