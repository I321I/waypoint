# UX Overhaul — 8 項 UX/行為修正設計

**日期**：2026-05-05
**狀態**：Draft，待 review 後進 writing-plans

## 動機

使用者在 Steam Deck 與一般桌面回報的 8 項問題與期望行為。整體目標：穩定 hotkey 行為一致性、簡化筆記編輯介面、加入即時 context 跟隨與位置記憶。

## 範圍

| # | 項目 | 風險 |
|---|---|---|
| 1 | Hotkey 連按一致性 | 中（並發 / state machine） |
| 2 | 移除 markdown toolbar | 低 |
| 3 | 新增右鍵功能選單 | 低 |
| 4 | Titlebar 顯示規則：全域加 globe icon、區域只顯示名稱 | 低 |
| 5 | 穿透模式隱藏 titlebar + 收起列表 | 中 |
| 6 | 隱藏 scrollbar（只筆記視窗） | 低 |
| 7 | 筆記視窗幾何（x/y/w/h）即時記憶 | 中 |
| 8 | 自動 context 切換跟隨 OS 前景視窗 | 高（需要 OS 事件 hook） |

---

## 1. Hotkey 連按一致性

### 問題

目前 `hotkey/mod.rs` 的 `register_hotkey` callback 會多次同時進入：兩次按鍵的 callback 都讀到 `list_open=true`，都 emit collapse-all-requested，第二次以後是 noop；或 callback 之間沒有同步，state 在 hide 還沒完成時被另一次 callback 看到 stale 值。觀察到的症狀是「只儲存關閉、偶爾儲存關閉→打開、或只關閉列表筆記沒關開」。

### 設計

加一個全域 `HOTKEY_INFLIGHT: Mutex<()>`（或 `AtomicBool`）。callback 入口：

```
let _guard = match HOTKEY_INFLIGHT.try_lock() {
    Ok(g) => g,
    Err(_) => { write_log_line("hotkey dropped: inflight"); return; }
};
```

拿到鎖後完整跑一個 cycle：

- **CollapseAll**：emit `waypoint://collapse-all-requested` → 等前端 ack（新 command `cmd_session_saved_ack`，寫一個 oneshot channel）或 100 ms 兜底 → `collapse_all_waypoint_windows` → `list_open=false`
- **OpenAll / OpenList**：直接呼叫對應 open，不需 ack

`_guard` drop 後鎖釋放。連按時第二次以後 try_lock 失敗就 drop。

### 為何 drop 而非 queue

連按多半是手抖或硬體 key repeat，使用者真實意圖只有一次。若要 round-trip 使用者再按一次（此時鎖已釋放），行為一致。

### 測試

- Rust unit：模擬連續呼叫 callback，斷言只有一次完整 cycle 跑完
- e2e（Linux）：xdotool 連發兩次 `ctrl+shift+space`，斷言 log 出現「hotkey fired」與「hotkey dropped: inflight」、session 檔案內仍含原本開的筆記列表（沒被 race 蓋掉）

---

## 2 + 3. 移除 markdown toolbar、新增右鍵功能選單

### 問題

目前 NoteWindow 上面有條 markdown 格式按鈕列（B/I/U/S/H1/H2/H3/list/task/code/⚙），佔空間。設定（字體大小、刪除）藏在 ⚙ 開的 SettingsPanel 裡。

### 設計

- 刪除：`Toolbar.svelte`、`SettingsPanel.svelte`、`NoteWindow.svelte` 內的 `<Toolbar>` 與 `<SettingsPanel>` 段落
- markdown 格式只剩 tiptap 內建鍵盤快捷鍵（`Cmd+B` / `Cmd+I` / `Cmd+Alt+1` 等）
- 新增 `note/ContextMenu.svelte`：絕對定位的浮動選單，在 Editor 區 `oncontextmenu` 顯示，項目固定四項：
  - **複製** — 呼叫 tiptap selection 的 markdown copy（保留格式）
  - **貼上** — `navigator.clipboard.readText` 後 `editor.commands.insertContent`
  - **字體大小** — 列入主選單，stepper：`−` 數字 `+`（沿用 SettingsPanel 同樣 logic）
  - **刪除此筆記** — 沿用 `ConfirmDialog`，呼叫 `notes.delete`

### UX 細節

- 點選單外或 Escape 關閉
- 沒有選取文字時「複製」灰掉
- titlebar 與 title `<input>` 上的右鍵不攔截，維持 OS 預設

### 測試

- Playwright render：右鍵 Editor 區，斷言 `ContextMenu` 渲染、四項可見、stepper 改值會 dispatch save、刪除走 ConfirmDialog
- e2e：略（無新 OS 互動）

---

## 4. Titlebar 顯示規則

### 問題

目前格式：`{title}-{contextId ?? "Global"}`。要改：

- 全域筆記（contextId 為 null）：`{title}` + globe icon
- 區域筆記：只顯示 `{title}`，不再加 context 字串

### 設計

`NoteWindow.svelte` 模板：

```svelte
<span class="note-title" data-tauri-drag-region>
  {title || "Untitled"}
  {#if contextId === null}<GlobeIcon />{/if}
</span>
```

GlobeIcon：inline 24×24 outlined SVG（Heroicons 風格），color = `var(--text-secondary)`。

### 測試

- Playwright render（既有 `NoteWindow.titlebar.render.test.pw.ts` 加分支）：全域筆記 → SVG 存在；區域筆記 → SVG 不存在、不出現「-」字

---

## 5. 穿透模式隱藏 chrome

### 問題

目前 passthrough 開啟僅啟用滑鼠穿透，視窗 chrome（titlebar + 按鈕列）仍顯示。要改成：純筆記視覺。

### 設計

NoteWindow.svelte：

```svelte
{#if !passthrough}
  <DraggableTitlebar>...</DraggableTitlebar>
{/if}
<div class="title-row"><input class="title-input" .../></div>
<div class="editor-area">...</div>
```

title `<input>` 維持顯示（使用者明確要求）。

進入穿透時：

- Rust 端 `cmd_toggle_passthrough_global` 在 emit `waypoint://passthrough-changed` 之外，再判斷「是否變成 ON」：
  - 是 → snapshot `state.list_window_open` 到 `state.pre_passthrough_list_open: AtomicBool`
  - emit `waypoint://passthrough-globally-on`：list webview 收到後自我 hide

退出穿透時：

- 若 `pre_passthrough_list_open=true` → `open_list_window(app)`

### 退穿透途徑

- 全域 hotkey `Ctrl+Shift+Q`（既有）
- tray 選單「穿透：開」項目（既有）
- 筆記內無視覺按鈕（titlebar 已隱藏）

### 測試

- Rust unit：toggle global on → state.pre_passthrough_list_open 反映上一個值；off → 自動 open list 條件正確
- Playwright render：passthrough=true 時 `.titlebar` 不存在、`.title-input` 仍存在
- e2e：toggle on/off round-trip 後 list 與筆記 chrome 狀態正確

---

## 6. 隱藏 scrollbar（筆記視窗）

### 設計

`NoteWindow.svelte` `<style>`：

```css
:global(.note-view) ::-webkit-scrollbar { width: 0; height: 0; }
:global(.note-view) * { scrollbar-width: none; }
.editor-area { overflow: auto; }
```

仍可捲（鍵盤、滑鼠滾輪、觸控板），只是視覺上看不到。list / settings / help 不動。

### 測試

- Playwright render：`getComputedStyle(scrollableEl)` 取 scrollbar 寬度 = 0；textarea 加長文後 scrollTop 仍可變

---

## 7. 筆記視窗幾何即時記憶

### 問題

目前位置/大小只有「按 ✕ 關閉」時隱性記憶（透過 session 還原）；resize 中斷或 Alt+F4 強關都會掉。

### 設計

`types.ts` 的 `NoteSettings` 加：

```ts
geometry?: { x: number; y: number; w: number; h: number };
```

`NoteWindow.svelte` onMount：

```ts
const win = getCurrentWindow();
const debouncedSave = debounce(async () => {
  const pos = await win.outerPosition();
  const size = await win.outerSize();
  const next = { ...note.settings, geometry: { x: pos.x, y: pos.y, w: size.width, h: size.height } };
  await notesApi.saveSettings(contextId, noteId, next);
}, 500);
unlistenMove = await win.onMoved(debouncedSave);
unlistenResize = await win.onResized(debouncedSave);
```

`hotkey::open_note_window` 開窗時：

```rust
let mut builder = WebviewWindowBuilder::new(...);
if let Some(geom) = note.settings.geometry {
    builder = builder.position(geom.x, geom.y).inner_size(geom.w, geom.h);
}
```

跨 context 共用一份（一個 note id 屬於一個 context，不會跨 context 重複）。

### 測試

- e2e：開筆記、`cmd_set_window_position(x=100,y=100)` → 等 600 ms（debounce + io）→ ✕ 關 → 重開 → 斷言 outer_position 為 (100, 100)
- 再加一個變體：resize → wait → Alt+F4（直接 close 不走 ✕）→ 重開 → 斷言尺寸保留

---

## 8. 自動 context 切換

### 問題

目前 `hotkey/mod.rs` 在 hotkey 按下時才呼叫 `get_focused_window()` 推導 context。使用者期待：alt-tab 切到別的 app 時，list 立刻反映新 context；切到 waypoint 自家視窗時維持先前 context。

### 設計

新增 `src-tauri/src/foreground_watcher/mod.rs`：

- 每平台一份 impl，公開介面：`pub fn start(app: AppHandle, on_change: impl Fn(WindowInfo) + Send + 'static)`
- Linux：`x11rb` 監聽 root window 的 `_NET_ACTIVE_WINDOW` PropertyNotify
- Windows：`SetWinEventHook(EVENT_SYSTEM_FOREGROUND, ...)`，event proc 將 hwnd 推到 channel
- macOS：`NSWorkspace.shared.notificationCenter.addObserver(name: didActivateApplicationNotification)`

收到事件流：

```
foreground event
  → 取對方 PID
  → if pid == process::id() → 忽略（自家視窗）
  → debounce 150 ms
  → derive_context_id(window_info, &config)
  → 寫入 state.active_context_id
  → emit "waypoint://active-context-changed" { contextId }
```

list webview 收 `active-context-changed`：

- 觸發 `flush-and-save-now`（讓現有筆記寫入磁碟）
- 收到 ack 後 hide 全部當前 context 的筆記（保留 session）
- 切到新 context、`load_session(new_context)`、依序 `open_note_window`
- list section 重 render 為新 context

事件式為主，輪詢只在事件式註冊失敗時 fallback（fallback 間隔 500 ms，記 log）。

### 測試

- Rust unit：模擬餵入 event（自家 PID / 外部 PID 各一份），斷言 active_context_id 變化正確、自家視窗不觸發切換
- Linux e2e：act 內 xvfb 無「真正前景應用程式」概念，跳過此 spec
- Windows e2e：用 PowerShell `(New-Object -ComObject WScript.Shell).AppActivate('notepad')` 觸發 foreground 事件，斷言 list 切到該 process 對應的 context

---

## 實作分階段（4 plan）

依風險與相依排序：

1. **Plan A — 純 UI**：item 2+3+4+6（toolbar 移除、右鍵選單、titlebar globe、scrollbar）。最小風險先行
2. **Plan B — 幾何記憶**：item 7。改 NoteSettings schema，獨立改避免跟其他改動撞
3. **Plan C — 穿透 chrome 隱藏**：item 5。動 lib.rs 事件流
4. **Plan D — Hotkey 序列化 + 自動 context 切換**：item 1+8。兩者都動 hotkey state machine 路徑與 OS 事件，合併避免重複改

每個 plan 結束都要：cargo test + npm test + npm run test:render 全綠 + act e2e-linux 全綠。

---

## 開放問題

- macOS foreground event 的 NSWorkspace observer 在 Tauri runtime 主執行緒上是否能正常執行（Tauri 的 event loop 與 NSApp 整合可能影響註冊時機）。發現問題時 fallback 為 100 ms 輪詢
- 字體大小 stepper 的最小/最大值沿用既有 8/32（SettingsPanel 之前的設定）
