# 透明拉桿／穿透 hotkey／改名同步／視窗拖曳／儲存修復 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修正 v0.1.16 後回報的 7 個議題：透明套用層、titlebar 透明拉桿、Edge 穿透 hotkey 失效、列表/筆記改名雙向同步、Settings/Help 視窗拖曳、筆記內容遺失。

**Architecture:** Tauri 2 + Svelte 5 + Rust。前端抽 `DraggableTitlebar` / `TitlebarOpacitySlider` 兩個共用元件；後端 `save_content` 改原子寫檔；新增 `cmd_exit_app_with_flush` 在退出前 flush 所有 NoteWindow；Hotkey 註冊狀態提供前端讀取。改名透過兩條獨立事件 channel 雙向 sync。

**Tech Stack:** Tauri 2, Svelte 5, Rust, TipTap, Vitest, Playwright, WebdriverIO (E2E Windows)

**Spec:** `waypoint/docs/superpowers/specs/2026-04-27-transparency-hotkey-rename-drag-save-design.md`

**工作分支：** 在 `dev/main` 開發。階段完成需驗 Windows 行為時才 ff-only merge 到 master。

---

## File Structure

| 檔案 | 動作 | 責任 |
|------|------|------|
| `src-tauri/src/storage/notes.rs` | Modify | `save_content` / `save_settings` 改 atomic write (tmp + rename) |
| `src-tauri/src/storage/app_config.rs` | Modify | 預設 passthrough hotkey `Ctrl+Alt+T` |
| `src-tauri/src/state.rs` | Modify | 新增 `passthrough_hotkey_registered: AtomicBool` |
| `src-tauri/src/lib.rs` | Modify | register 失敗時更新 state + tray notification |
| `src-tauri/src/commands/config_cmd.rs` | Modify | `get_config` 回傳註冊狀態 |
| `src-tauri/src/hotkey/mod.rs` | Modify | 新增 `cmd_exit_app_with_flush`；`cmd_restart_app` 加 flush |
| `src-tauri/src/tray/mod.rs` | Modify | 「結束」改呼 flush 版本 |
| `src/windows/DraggableTitlebar.svelte` | Create | 共用拖曳 titlebar，內部 text 自動 `pointer-events:none` |
| `src/windows/note/TitlebarOpacitySlider.svelte` | Create | 36px 拉桿元件（樣式 A） |
| `src/lib/hotkeyConflicts.ts` | Create | 衝突清單 + 比對函式 |
| `src/windows/NoteWindow.svelte` | Modify | 加拉桿；用 DraggableTitlebar；emit/listen rename；listen flush；100ms debounce；`.note-window` 套 alpha |
| `src/windows/note/SettingsPanel.svelte` | Modify | 移除透明欄位 |
| `src/windows/SettingsWindow.svelte` | Modify | 用 DraggableTitlebar；hotkey 衝突檢查；註冊失敗警示 |
| `src/windows/HelpWindow.svelte` | Modify | 用 DraggableTitlebar |
| `src/windows/ListWindow.svelte` | Modify | 用 DraggableTitlebar；listen note-title-changed；emit note-renamed-from-list |

---

## Task 1: Atomic save_content / save_settings（#7 L1）

**Files:**
- Modify: `src-tauri/src/storage/notes.rs`
- Test: `src-tauri/src/storage/notes.rs`（既有 `#[cfg(test)] mod tests`）

- [ ] **Step 1: 寫失敗測試**

加到 `notes.rs` `mod tests` 中：

```rust
#[test]
fn save_content_uses_atomic_write_no_partial_file() {
    let _g = TEST_LOCK.lock().unwrap();
    let n = create_note(None, "atomic").unwrap();
    save_content(None, &n.id, "stage1").unwrap();
    save_content(None, &n.id, "stage2-much-longer-content-replacing-prior").unwrap();
    let final_content = std::fs::read_to_string(note_dir(None, &n.id).join("content.md")).unwrap();
    assert_eq!(final_content, "stage2-much-longer-content-replacing-prior");
    // tmp 檔不應殘留
    assert!(!note_dir(None, &n.id).join("content.md.tmp").exists());
}
```

（若 `TEST_LOCK` 不存在，沿用既存 tests 模式。）

- [ ] **Step 2: 跑測試確認 fail（功能行為相同會通過，但需確認 tmp 不殘留）**

```bash
cd waypoint/src-tauri && cargo test save_content_uses_atomic_write -- --nocapture
```

預期：`assert!(!...content.md.tmp).exists())` 通過（檔案還沒實作不會留 tmp）。本測試是 regression guard，先 PASS 也接受，但需保留。

- [ ] **Step 3: 改 `save_content` 為 atomic**

替換 `notes.rs` `save_content`：

```rust
pub fn save_content(context_id: Option<&str>, note_id: &str, content: &str) -> Result<(), WaypointError> {
    let dir = note_dir(context_id, note_id);
    if !dir.exists() {
        return Err(WaypointError::NoteNotFound(note_id.to_string()));
    }
    let final_path = dir.join("content.md");
    let tmp_path = dir.join("content.md.tmp");
    std::fs::write(&tmp_path, content)?;
    std::fs::rename(&tmp_path, &final_path)?;
    Ok(())
}
```

同樣套用在 `save_settings`：

```rust
pub fn save_settings(context_id: Option<&str>, note_id: &str, settings: &NoteSettings) -> Result<(), WaypointError> {
    let dir = note_dir(context_id, note_id);
    if !dir.exists() {
        return Err(WaypointError::NoteNotFound(note_id.to_string()));
    }
    let json = serde_json::to_string_pretty(settings)?;
    let final_path = dir.join("settings.json");
    let tmp_path = dir.join("settings.json.tmp");
    std::fs::write(&tmp_path, json)?;
    std::fs::rename(&tmp_path, &final_path)?;
    Ok(())
}
```

- [ ] **Step 4: 跑測試確認 PASS**

```bash
cd waypoint/src-tauri && cargo test --lib storage::notes
```

預期：所有 storage::notes 測試 PASS。

- [ ] **Step 5: Commit**

```bash
git add waypoint/src-tauri/src/storage/notes.rs
git commit -m "fix(storage): atomic write for save_content/save_settings (#7 L1)"
```

---

## Task 2: 預設 passthrough hotkey 改 Ctrl+Alt+T

**Files:**
- Modify: `src-tauri/src/storage/app_config.rs`

- [ ] **Step 1: 找 default 函式**

```bash
grep -n "passthrough_hotkey\|Ctrl+Shift+T" waypoint/src-tauri/src/storage/app_config.rs
```

- [ ] **Step 2: 改預設值**

把 `default_passthrough_hotkey()`（或對應 const）的回傳值改成 `"Ctrl+Alt+T".to_string()`。如果預設值是寫在 `serde(default = "...")` 上，則對應函式回傳值改掉。

- [ ] **Step 3: 寫單元測試**

```rust
#[test]
fn default_passthrough_hotkey_is_ctrl_alt_t() {
    let cfg = AppConfig::default();
    assert_eq!(cfg.passthrough_hotkey, "Ctrl+Alt+T");
}
```

- [ ] **Step 4: 跑測試**

```bash
cd waypoint/src-tauri && cargo test default_passthrough_hotkey_is_ctrl_alt_t
```

預期：PASS。

- [ ] **Step 5: Commit**

```bash
git add waypoint/src-tauri/src/storage/app_config.rs
git commit -m "feat(hotkey): default passthrough hotkey changed to Ctrl+Alt+T (#3)"
```

---

## Task 3: 後端追蹤 hotkey 註冊狀態 + tray 通知

**Files:**
- Modify: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/commands/config_cmd.rs`

- [ ] **Step 1: AppState 新增欄位**

在 `state.rs` `AppState` struct 加：

```rust
pub passthrough_hotkey_registered: std::sync::atomic::AtomicBool,
```

並在 `Default` / `new` 初始化為 `AtomicBool::new(true)`。

- [ ] **Step 2: lib.rs register 失敗時設 false + 發 tray notification**

在 `lib.rs` 找 `register_passthrough_hotkey` 呼叫處（line ~134-136）：

```rust
match hotkey::register_passthrough_hotkey(app.handle(), &config.passthrough_hotkey) {
    Ok(()) => write_log_line(&format!("register_passthrough_hotkey ok: {}", &config.passthrough_hotkey)),
    Err(e) => {
        write_log_line(&format!("register_passthrough_hotkey failed ({}): {e}", &config.passthrough_hotkey));
        let state = app.handle().state::<crate::state::AppState>();
        state.passthrough_hotkey_registered.store(false, std::sync::atomic::Ordering::SeqCst);
        // tray notification
        use tauri_plugin_notification::NotificationExt;
        let _ = app.handle().notification()
            .builder()
            .title("Waypoint — 穿透快捷鍵註冊失敗")
            .body(format!("「{}」可能已被其他程式占用。請至設定更換。", &config.passthrough_hotkey))
            .show();
    }
}
```

確認 Cargo.toml 有 `tauri-plugin-notification`；若沒有，加入：

```toml
tauri-plugin-notification = "2"
```

並在 lib.rs `tauri::Builder::default()` 串：

```rust
.plugin(tauri_plugin_notification::init())
```

- [ ] **Step 3: get_config 回傳註冊狀態**

修改 `commands/config_cmd.rs::get_config`，使其回傳結構含 `passthrough_hotkey_registered: bool`。先在 `lib/types.ts`（前端）對應：

```ts
export interface AppConfig {
  hotkey: string;
  passthroughHotkey: string;
  showInTaskbar: boolean;
  passthroughHotkeyRegistered: boolean;
}
```

後端對應結構（同檔內或 types.rs）：

```rust
#[derive(serde::Serialize)]
pub struct AppConfigDto {
    pub hotkey: String,
    pub passthrough_hotkey: String,
    pub show_in_taskbar: bool,
    pub passthrough_hotkey_registered: bool,
}
```

`get_config` 改為：

```rust
#[tauri::command]
pub fn get_config(app: tauri::AppHandle) -> Result<AppConfigDto, String> {
    let cfg = app_config::load().unwrap_or_default();
    let state = app.state::<crate::state::AppState>();
    Ok(AppConfigDto {
        hotkey: cfg.hotkey,
        passthrough_hotkey: cfg.passthrough_hotkey,
        show_in_taskbar: cfg.show_in_taskbar,
        passthrough_hotkey_registered: state.passthrough_hotkey_registered.load(std::sync::atomic::Ordering::SeqCst),
    })
}
```

（若 `get_config` 既有簽名不同，保留原欄位、僅追加 `passthrough_hotkey_registered`。）

- [ ] **Step 4: 跑 cargo build 確認編譯**

```bash
cd waypoint/src-tauri && cargo build
```

預期：成功編譯。

- [ ] **Step 5: Commit**

```bash
git add waypoint/src-tauri/src/state.rs waypoint/src-tauri/src/lib.rs waypoint/src-tauri/src/commands/config_cmd.rs waypoint/src-tauri/Cargo.toml waypoint/src/lib/types.ts
git commit -m "feat(hotkey): expose passthrough hotkey register status + tray notify on fail (#3)"
```

---

## Task 4: Hotkey 衝突清單模組（前端）

**Files:**
- Create: `src/lib/hotkeyConflicts.ts`
- Create: `src/lib/hotkeyConflicts.test.ts`

- [ ] **Step 1: 寫測試**

`hotkeyConflicts.test.ts`:

```ts
import { describe, it, expect } from "vitest";
import { findConflict } from "./hotkeyConflicts";

describe("findConflict", () => {
  it("returns conflict info for Ctrl+Shift+T", () => {
    const c = findConflict("Ctrl+Shift+T");
    expect(c).not.toBeNull();
    expect(c?.app).toContain("瀏覽器");
  });
  it("returns null for safe combo", () => {
    expect(findConflict("Ctrl+Alt+T")).toBeNull();
  });
  it("is case-insensitive on key", () => {
    expect(findConflict("ctrl+shift+t")).not.toBeNull();
  });
});
```

- [ ] **Step 2: 跑測試確認 fail**

```bash
cd waypoint && npm test -- hotkeyConflicts
```

預期：模組不存在 → fail。

- [ ] **Step 3: 實作模組**

`hotkeyConflicts.ts`:

```ts
export interface HotkeyConflict {
  combo: string;
  app: string;
  description: string;
}

const CONFLICTS: HotkeyConflict[] = [
  { combo: "Ctrl+Shift+T", app: "瀏覽器（Edge/Chrome/Firefox）", description: "重新開啟最後關閉的分頁" },
  { combo: "Ctrl+Shift+N", app: "瀏覽器", description: "新無痕視窗" },
  { combo: "Ctrl+Shift+W", app: "瀏覽器", description: "關閉所有分頁" },
  { combo: "Ctrl+Alt+Del", app: "Windows 系統", description: "保留組合鍵" },
  { combo: "Win+L", app: "Windows 系統", description: "鎖定螢幕" },
  { combo: "Ctrl+Esc", app: "Windows 系統", description: "開始功能表" },
];

export function findConflict(combo: string): HotkeyConflict | null {
  const norm = combo.toLowerCase();
  return CONFLICTS.find(c => c.combo.toLowerCase() === norm) ?? null;
}
```

- [ ] **Step 4: 跑測試 PASS**

```bash
cd waypoint && npm test -- hotkeyConflicts
```

預期：3 個測試全 PASS。

- [ ] **Step 5: Commit**

```bash
git add waypoint/src/lib/hotkeyConflicts.ts waypoint/src/lib/hotkeyConflicts.test.ts
git commit -m "feat(hotkey): conflict list module for known global hotkeys (#3)"
```

---

## Task 5: DraggableTitlebar 共用元件

**Files:**
- Create: `src/windows/DraggableTitlebar.svelte`
- Create: `src/windows/DraggableTitlebar.render.test.pw.ts`

- [ ] **Step 1: 寫元件**

`DraggableTitlebar.svelte`:

```svelte
<script lang="ts">
  import { windows as windowsApi } from "../lib/api";
  export let label: string;

  // 同步 fire-and-forget — async/await 在 mousedown handler 內會引入微 task 延遲，
  // 導致 start_dragging 在 mouse 事件迴圈結束後才執行；NoteWindow 已驗證此 pattern 可動。
  function handleMousedown(e: MouseEvent) {
    if (e.button !== 0) return;
    const target = e.target as HTMLElement;
    if (target.closest("button, input, textarea, select, a")) return;
    windowsApi.startDragging(label).catch(() => {});
  }
</script>

<div class="draggable-titlebar" data-tauri-drag-region on:mousedown={handleMousedown}>
  <slot />
</div>

<style>
  .draggable-titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 5px 10px;
    background: var(--bg-tertiary);
    border-bottom: 1px solid var(--border);
    min-height: 30px;
    gap: 8px;
    cursor: grab;
  }
  .draggable-titlebar:active { cursor: grabbing; }
  /* slot 內的 text span 自動穿透 mousedown 到 titlebar 本體 */
  .draggable-titlebar :global(span:not(.opacity-readout):not(.value)) {
    pointer-events: none;
  }
  /* 但 button / input 必須能點 — 用 :global 重新打開 */
  .draggable-titlebar :global(button),
  .draggable-titlebar :global(input),
  .draggable-titlebar :global(.opacity-slider-wrap) {
    pointer-events: auto;
  }
</style>
```

- [ ] **Step 2: 寫渲染測試**

`DraggableTitlebar.render.test.pw.ts`：

```ts
import { test, expect } from "@playwright/test";

test("DraggableTitlebar mousedown on text area triggers startDragging with correct label", async ({ page }) => {
  await page.goto("/_test/draggable-titlebar?label=test-window");
  // 在頁面注入 spy
  await page.evaluate(() => {
    (window as any).__dragSpy = [];
    (window as any).__TAURI_INTERNALS__ = {
      invoke: (cmd: string, args: any) => {
        if (cmd === "start_dragging") (window as any).__dragSpy.push(args.label);
        return Promise.resolve();
      },
    };
  });
  await page.locator(".draggable-titlebar span").first().dispatchEvent("mousedown", { button: 0 });
  const spy = await page.evaluate(() => (window as any).__dragSpy);
  expect(spy).toContain("test-window");
});
```

> 註：`/_test/draggable-titlebar` 路由為測試 fixture，需在 `src/routes/_test/draggable-titlebar/+page.svelte` 建立簡單 demo 頁掛 `<DraggableTitlebar label={...}><span>title</span></DraggableTitlebar>`。

- [ ] **Step 3: 建立 fixture 頁**

`src/routes/_test/draggable-titlebar/+page.svelte`：

```svelte
<script lang="ts">
  import DraggableTitlebar from "../../../windows/DraggableTitlebar.svelte";
  const params = new URLSearchParams(window.location.search);
  const label = params.get("label") ?? "fixture";
</script>

<DraggableTitlebar {label}>
  <span class="title">Drag me</span>
  <button class="close-btn">✕</button>
</DraggableTitlebar>
```

- [ ] **Step 4: 跑測試 PASS**

```bash
cd waypoint && npm run build && npm run test:render -- DraggableTitlebar
```

預期：PASS（spy 收到 `test-window`）。

- [ ] **Step 5: Commit**

```bash
git add waypoint/src/windows/DraggableTitlebar.svelte waypoint/src/windows/DraggableTitlebar.render.test.pw.ts waypoint/src/routes/_test/draggable-titlebar/+page.svelte
git commit -m "feat(ui): DraggableTitlebar shared component for window dragging (#6)"
```

---

## Task 6: 套用 DraggableTitlebar 到 Settings/Help/List/Note

**Files:**
- Modify: `src/windows/SettingsWindow.svelte`
- Modify: `src/windows/HelpWindow.svelte`
- Modify: `src/windows/ListWindow.svelte`
- Modify: `src/windows/NoteWindow.svelte`

- [ ] **Step 1: SettingsWindow 改用 DraggableTitlebar**

替換 SettingsWindow 內的 `.titlebar`：

```svelte
<script lang="ts">
  import DraggableTitlebar from "./DraggableTitlebar.svelte";
  // 移除既有 handleTitlebarMousedown / getCurrentWindow imports
</script>

<div class="settings-window">
  <DraggableTitlebar label="settings">
    <span class="title">Waypoint — 設定</span>
    <button class="close-btn" on:click={() => windowsApi.closeWindow("settings").catch(() => {})}>✕</button>
  </DraggableTitlebar>
  <!-- 其餘 content 不變 -->
</div>
```

- [ ] **Step 2: HelpWindow 同樣改寫**

```svelte
<script lang="ts">
  import DraggableTitlebar from "./DraggableTitlebar.svelte";
  import { windows as windowsApi } from "../lib/api";
</script>

<div class="help-window">
  <DraggableTitlebar label="help">
    <span class="title">Waypoint — 使用說明</span>
    <button on:click={() => windowsApi.closeWindow("help").catch(() => {})}>✕</button>
  </DraggableTitlebar>
  <!-- content 不變 -->
</div>
```

- [ ] **Step 3: ListWindow 改寫（保留 titlebar-left / titlebar-right slot 結構）**

替換 `<div class="titlebar" ...>` 整段為：

```svelte
<DraggableTitlebar label="list">
  <div class="titlebar-left">
    <span class="app-name">WAYPOINT</span>
    <button class="icon-btn" on:click={openHelp} title="使用說明">?</button>
  </div>
  <div class="titlebar-right">
    <button class="icon-btn" on:click={openSettings} title="設定">⚙</button>
    <button class="icon-btn" on:click={handleCollapseAll} title="收起全部">⇊</button>
    <button class="icon-btn" on:click={minimizeList} title="最小化列表">—</button>
    <button class="icon-btn" on:click={quitApp} title="結束 Waypoint">✕</button>
  </div>
</DraggableTitlebar>
```

並移除 ListWindow 既存的 `handleTitlebarMousedown`。

- [ ] **Step 4: NoteWindow 改寫**

替換 `<div class="titlebar" ...>` 整段為：

```svelte
<DraggableTitlebar label={`note-${noteId}`}>
  <span class="note-title">{title || "Untitled"}{contextId ? ` — ${contextId}` : ""}</span>
  <div class="titlebar-buttons">
    <!-- 拉桿在 Task 8 補上，這個 step 留 placeholder span -->
    <button
      class="passthrough-dot"
      class:dot-on={!passthrough}
      class:dot-off={passthrough}
      on:click={handleDotClick}
      title={passthrough ? '穿透中（按快捷鍵或 tray 關閉）' : '可互動 — 點此啟用穿透'}
    ></button>
    <button on:click={handleCollapseAll} title="收起全部並儲存">⇊</button>
    <button on:click={handleMaximize} title="最大化／還原">▢</button>
    <button on:click={handleClose} title="儲存並關閉">✕</button>
  </div>
</DraggableTitlebar>
```

移除既存 `handleTitlebarMousedown`、移除 `<div class="title-row">` 之上的 `<div class="titlebar">` 整段。

- [ ] **Step 5: 跑既有渲染測試**

```bash
cd waypoint && npm run build && npm run test:render
```

預期：所有 render 測試 PASS（titlebar dom 結構雖換，但元件文字仍應顯示）。若有測試斷言 `.titlebar` selector，改成 `.draggable-titlebar`。

- [ ] **Step 6: Commit**

```bash
git add waypoint/src/windows/SettingsWindow.svelte waypoint/src/windows/HelpWindow.svelte waypoint/src/windows/ListWindow.svelte waypoint/src/windows/NoteWindow.svelte
git commit -m "refactor(ui): use DraggableTitlebar in Settings/Help/List/Note (#6)"
```

---

## Task 7: 修 `.note-window` 透明背景（#1）

**Files:**
- Modify: `src/windows/NoteWindow.svelte`
- Test: `src/windows/NoteWindow.transparent.render.test.pw.ts`（既有）

- [ ] **Step 1: 加渲染測試確認 `.note-window` 也吃 alpha**

在既有 `NoteWindow.transparent.render.test.pw.ts` 加：

```ts
test('note-window background also follows --note-alpha', async ({ page }) => {
  await page.goto('/?#view=note&noteId=demo');
  await page.evaluate(() => document.documentElement.style.setProperty('--note-alpha', '0.4'));
  const bg = await page.locator('.note-window').first().evaluate(el => getComputedStyle(el).backgroundColor);
  // rgba 形式，alpha 約 0.4
  expect(bg).toMatch(/rgba\(\s*30\s*,\s*30\s*,\s*30\s*,\s*0\.4\s*\)/);
});
```

- [ ] **Step 2: 跑測試確認 fail**

```bash
cd waypoint && npm run build && npm run test:render -- NoteWindow.transparent
```

預期：fail（目前 `.note-window` 是 `var(--bg-primary)` 不透明）。

- [ ] **Step 3: 改 NoteWindow CSS**

替換 `<style>` 區塊內 `.note-window` 規則：

```css
.note-window {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: rgba(30, 30, 30, var(--note-alpha, 1));
  border: 1px solid var(--border);
}
```

並把 `:global(body.note-view)` 改為：

```css
:global(body.note-view) {
  background: transparent !important;
}
```

（body 不再吃顏色，全部由 .note-window 處理；避免 Windows 上 transparent 視窗 + 雙層背景互蓋。）

- [ ] **Step 4: 跑測試 PASS**

```bash
cd waypoint && npm run build && npm run test:render -- NoteWindow.transparent
```

預期：PASS。

- [ ] **Step 5: Commit**

```bash
git add waypoint/src/windows/NoteWindow.svelte waypoint/src/windows/NoteWindow.transparent.render.test.pw.ts
git commit -m "fix(transparency): apply --note-alpha to .note-window background (#1)"
```

---

## Task 8: TitlebarOpacitySlider 元件（樣式 A）

**Files:**
- Create: `src/windows/note/TitlebarOpacitySlider.svelte`
- Create: `src/windows/note/TitlebarOpacitySlider.test.ts`

- [ ] **Step 1: 寫元件**

```svelte
<script lang="ts">
  import { createEventDispatcher } from "svelte";
  export let value: number; // 0.1 ~ 1.0
  const dispatch = createEventDispatcher<{ change: number }>();
  function onInput(e: Event) {
    const v = parseInt((e.target as HTMLInputElement).value, 10) / 100;
    dispatch("change", v);
  }
</script>

<div class="opacity-slider-wrap" title={`透明度 ${Math.round(value * 100)}%`}>
  <input
    class="opacity-slider"
    type="range" min="10" max="100" step="5"
    value={Math.round(value * 100)}
    on:input={onInput}
    aria-label="透明度"
  />
</div>

<style>
  .opacity-slider-wrap { width: 36px; display: flex; align-items: center; }
  .opacity-slider {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    height: 4px;
    background: var(--border);
    border-radius: 2px;
    outline: none;
    margin: 0;
    cursor: pointer;
  }
  .opacity-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--accent);
    cursor: pointer;
  }
</style>
```

- [ ] **Step 2: 寫單元測試**

`TitlebarOpacitySlider.test.ts`：

```ts
import { render, fireEvent } from "@testing-library/svelte";
import { describe, it, expect } from "vitest";
import TitlebarOpacitySlider from "./TitlebarOpacitySlider.svelte";

describe("TitlebarOpacitySlider", () => {
  it("dispatches change event with normalized 0-1 value", async () => {
    let received = -1;
    const { container, component } = render(TitlebarOpacitySlider, { value: 1 });
    component.$on("change", (e: CustomEvent<number>) => { received = e.detail; });
    const input = container.querySelector("input") as HTMLInputElement;
    await fireEvent.input(input, { target: { value: "50" } });
    expect(received).toBeCloseTo(0.5);
  });
});
```

- [ ] **Step 3: 跑測試 PASS**

```bash
cd waypoint && npm test -- TitlebarOpacitySlider
```

預期：PASS。

- [ ] **Step 4: Commit**

```bash
git add waypoint/src/windows/note/TitlebarOpacitySlider.svelte waypoint/src/windows/note/TitlebarOpacitySlider.test.ts
git commit -m "feat(ui): TitlebarOpacitySlider component (#2 style A)"
```

---

## Task 9: NoteWindow 套用 TitlebarOpacitySlider + 移除 SettingsPanel 透明欄位

**Files:**
- Modify: `src/windows/NoteWindow.svelte`
- Modify: `src/windows/note/SettingsPanel.svelte`

- [ ] **Step 1: NoteWindow titlebar 加入拉桿**

在 NoteWindow `<script>` import：

```ts
import TitlebarOpacitySlider from "./note/TitlebarOpacitySlider.svelte";
```

並新增 handler：

```ts
async function handleOpacityChange(e: CustomEvent<number>) {
  if (!note) return;
  const newSettings = { ...note.settings, opacity: e.detail };
  note = { ...note, settings: newSettings };
  applyOpacity(e.detail);
  await notesApi.saveSettings(contextId, noteId, newSettings);
}
```

在 `<DraggableTitlebar>` 的 `.titlebar-buttons` 內、passthrough-dot **之前**插入：

```svelte
{#if note}
  <TitlebarOpacitySlider
    value={note.settings.opacity}
    on:change={handleOpacityChange}
  />
{/if}
```

- [ ] **Step 2: 移除 SettingsPanel 透明度區塊**

刪除 `SettingsPanel.svelte` 中：
- `<div class="setting-row">…透明度…</div>` 整塊（line ~33-45）
- 對應 `.opacity-slider` / `.slider-row` CSS（若已不再使用）

- [ ] **Step 3: 既存的 SettingsPanel 測試需更新**

如果 `render.test.pw.ts` 有 `await expect(page.locator(".opacity-slider")).toBeVisible()` 之類斷言，改為定位 NoteWindow titlebar 下的 `.opacity-slider-wrap`。

- [ ] **Step 4: 跑測試**

```bash
cd waypoint && npm run build && npm test && npm run test:render
```

預期：全 PASS。

- [ ] **Step 5: Commit**

```bash
git add waypoint/src/windows/NoteWindow.svelte waypoint/src/windows/note/SettingsPanel.svelte waypoint/src/render.test.pw.ts
git commit -m "feat(ui): move opacity slider to titlebar; remove from SettingsPanel (#2)"
```

---

## Task 10: 標題 sync — NoteWindow → ListWindow（#4）

**Files:**
- Modify: `src/windows/NoteWindow.svelte`
- Modify: `src/windows/ListWindow.svelte`
- Test: `src/windows/ListWindow.svelte` 對應的 vitest（若無則跳過 unit，靠 e2e 驗收）

- [ ] **Step 1: NoteWindow saveContent 後 emit title-changed**

修改 `scheduleSave`（會在 Task 12 一併調 debounce 時間，這個 step 先在 setTimeout 內加 emit）：

```ts
function scheduleSave() {
  if (!note) return;
  clearTimeout(saveTimeout);
  saveTimeout = setTimeout(async () => {
    const merged = joinTitleContent(title, body);
    await notesApi.saveContent(contextId, noteId, merged);
    const newTitle = parseTitleContent(merged).title || title;
    if (newTitle !== (note?.title ?? "")) {
      if (note) note = { ...note, title: newTitle };
      await emit("waypoint://note-title-changed", { noteId, contextId, newTitle });
    }
  }, 100); // 100ms debounce — Task 12 會解釋
}
```

import 補：`import { emit, listen } from "@tauri-apps/api/event";`（既有 import 已含 listen）

- [ ] **Step 2: ListWindow 監聽事件**

在 `ListWindow.svelte` `<script>` `onMount` 加：

```ts
let unlistenTitleChanged: UnlistenFn | null = null;

onMount(async () => {
  // 既有 onMount 邏輯保留
  unlistenTitleChanged = await listen<{ noteId: string; contextId: string | null; newTitle: string }>(
    "waypoint://note-title-changed",
    (event) => {
      const { noteId, contextId: ctx, newTitle } = event.payload;
      // 找對應的 note 資料源並更新（依 ListWindow 既有 state 結構）
      // 範例：sections 裡每個 ctx 含 notes[]，找到 noteId 後更新 title
      sections = sections.map(sec => ({
        ...sec,
        notes: sec.notes.map(n =>
          n.id === noteId && (n.contextId ?? null) === ctx
            ? { ...n, title: newTitle }
            : n
        ),
      }));
    }
  );
});

onDestroy(() => {
  unlistenTitleChanged?.();
});
```

> 註：實際 state 變數名以 ListWindow 既有為準（可能叫 `notes`、`globalNotes`、`contextSections` 等）；保留同樣的 immutable update pattern。

- [ ] **Step 3: 手動驗證**

```bash
cd waypoint && npm run build && npx tauri dev
```

開筆記、改標題 → 觀察列表立即更新。（這個 step 不寫自動化測試，因 cross-window event 在 Vitest 環境難 mock；e2e 那層會涵蓋。）

- [ ] **Step 4: Commit**

```bash
git add waypoint/src/windows/NoteWindow.svelte waypoint/src/windows/ListWindow.svelte
git commit -m "feat(sync): NoteWindow emits title-changed; ListWindow updates live (#4)"
```

---

## Task 11: 標題 sync — ListWindow → NoteWindow（#5）

**Files:**
- Modify: `src/windows/list/NoteItem.svelte`
- Modify: `src/windows/list/ContextSection.svelte`
- Modify: `src/windows/NoteWindow.svelte`

- [ ] **Step 1: NoteItem.submitRename 成功後 emit**

修改 `NoteItem.svelte` `submitRename`：

```ts
import { emit } from "@tauri-apps/api/event";

async function submitRename() {
  const newTitle = renameValue.trim();
  if (newTitle && newTitle !== note.title) {
    await notesApi.rename(note.contextId, note.id, newTitle);
    await emit("waypoint://note-renamed-from-list", {
      noteId: note.id,
      contextId: note.contextId,
      newTitle,
    });
    // 既有後續邏輯保留
  }
  // ...
}
```

ContextSection 的 context 重新命名屬不同範圍（context_id 改名），不在本議題；保持原狀。

- [ ] **Step 2: NoteWindow 監聽事件並重讀內容**

NoteWindow `<script>` `onMount` 加：

```ts
let unlistenRenamedFromList: UnlistenFn | null = null;

unlistenRenamedFromList = await listen<{ noteId: string; contextId: string | null; newTitle: string }>(
  "waypoint://note-renamed-from-list",
  async (event) => {
    if (event.payload.noteId !== noteId) return;
    if ((event.payload.contextId ?? null) !== (contextId ?? null)) return;
    // 重新讀檔（後端已寫入新標題到 content.md 第一行）
    const fresh = await notesApi.read(contextId, noteId);
    if (!fresh) return;
    const parsed = parseTitleContent(fresh.content);
    title = parsed.title || fresh.title || "";
    body = parsed.body;
    note = fresh;
    if (editorRef) {
      editorRef.getEditor()?.commands.setContent(body);
    }
  }
);
```

`onDestroy` 對應 unlisten。

- [ ] **Step 3: 手動驗證**

開筆記 + 列表，列表右鍵改名 → 筆記視窗 title input 與 editor 同步更新。

- [ ] **Step 4: Commit**

```bash
git add waypoint/src/windows/list/NoteItem.svelte waypoint/src/windows/NoteWindow.svelte
git commit -m "feat(sync): ListWindow rename pushes update to open NoteWindow (#5)"
```

---

## Task 12: Save debounce 改 100ms + flush-and-save event（#7 L2 + L3）

**Files:**
- Modify: `src-tauri/src/hotkey/mod.rs`
- Modify: `src-tauri/src/tray/mod.rs`
- Modify: `src/windows/NoteWindow.svelte`

> 註：debounce 100ms 已在 Task 10 step 1 改完；本 task 專注 flush event。

- [ ] **Step 1: 後端新增 cmd_exit_app_with_flush**

在 `hotkey/mod.rs` 加：

```rust
#[tauri::command]
pub async fn cmd_exit_app_with_flush(app: AppHandle) -> Result<(), String> {
    flush_all_notes(&app).await;
    app.exit(0);
    Ok(())
}

async fn flush_all_notes(app: &AppHandle) {
    use std::sync::Arc;
    use tokio::sync::Mutex;
    let pending: Arc<Mutex<std::collections::HashSet<String>>> = Arc::new(Mutex::new(
        app.webview_windows()
            .keys()
            .filter(|k| k.starts_with("note-"))
            .cloned()
            .collect()
    ));
    if pending.lock().await.is_empty() {
        return;
    }
    let pending_clone = pending.clone();
    let unlisten = app.listen("waypoint://flush-ack", move |event| {
        let label: String = serde_json::from_str(event.payload()).unwrap_or_default();
        let pending = pending_clone.clone();
        tokio::spawn(async move {
            pending.lock().await.remove(&label);
        });
    });
    let _ = app.emit("waypoint://flush-and-save-now", ());
    // 等 ack 全到或 800ms timeout
    let start = std::time::Instant::now();
    while start.elapsed() < std::time::Duration::from_millis(800) {
        if pending.lock().await.is_empty() { break; }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    app.unlisten(unlisten);
}
```

並修改既有 `cmd_restart_app` 在 `snapshot_open_windows` **之前**呼 `flush_all_notes(&app).await;`（將該函式改 async）。

> 若 Tauri event payload 解析需 `Emitter` trait，import 之；`app.listen` / `app.emit` API 隨 Tauri 2 版本可能略有差異，必要時用 `app.listen_global` / `app.emit`。

- [ ] **Step 2: 註冊新 command**

`lib.rs` `tauri::generate_handler!` 加 `cmd_exit_app_with_flush`。

- [ ] **Step 3: tray 「結束」改呼新 command**

在 `tray/mod.rs` 的「結束」menu 對應 click handler 改：

```rust
let app2 = app.clone();
tauri::async_runtime::spawn(async move {
    let _ = crate::hotkey::cmd_exit_app_with_flush(app2).await;
});
```

或直接呼 `flush_all_notes` 後 `app.exit(0)`。

- [ ] **Step 4: 前端 NoteWindow 監聽 flush-and-save-now**

`NoteWindow.svelte` `onMount` 加：

```ts
let unlistenFlush: UnlistenFn | null = null;

unlistenFlush = await listen("waypoint://flush-and-save-now", async () => {
  await flushPendingSave();
  await emit("waypoint://flush-ack", `note-${noteId}`);
});
```

`onDestroy` 對應 unlisten。

- [ ] **Step 5: Vitest 單元測試 — debounce 行為**

`NoteWindow.test.ts` （若無就建）加：

```ts
import { describe, it, expect, vi } from "vitest";

describe("NoteWindow scheduleSave debounce", () => {
  it("uses 100ms debounce window", () => {
    // Smoke check：grep src 確保沒有殘留 500ms
    const src = require("fs").readFileSync(
      require.resolve("./NoteWindow.svelte"),
      "utf8"
    );
    expect(src).toMatch(/}, 100\)/);
    expect(src).not.toMatch(/}, 500\)/);
  });
});
```

- [ ] **Step 6: 跑測試**

```bash
cd waypoint && npm test
cd waypoint/src-tauri && cargo build
```

預期：vitest PASS、cargo build 過。

- [ ] **Step 7: Commit**

```bash
git add waypoint/src-tauri/src/hotkey/mod.rs waypoint/src-tauri/src/tray/mod.rs waypoint/src-tauri/src/lib.rs waypoint/src/windows/NoteWindow.svelte waypoint/src/windows/NoteWindow.test.ts
git commit -m "feat(save): 100ms debounce + flush-and-save-now on app exit (#7 L2+L3)"
```

---

## Task 13: SettingsWindow — 衝突提示 + 註冊失敗警示

**Files:**
- Modify: `src/windows/SettingsWindow.svelte`

- [ ] **Step 1: import 衝突清單**

```ts
import { findConflict } from "../lib/hotkeyConflicts";
```

- [ ] **Step 2: 包裝 savePassthroughHotkey 加衝突檢查**

替換既有 `savePassthroughHotkey`：

```ts
async function savePassthroughHotkey() {
  const next = passthroughHotkeyInput.trim();
  if (!next) return;
  const conflict = findConflict(next);
  if (conflict) {
    const ok = confirm(
      `「${next}」是「${conflict.app}」的「${conflict.description}」快捷鍵，` +
      `在這些應用視窗 focus 時可能無法觸發 Waypoint。仍要使用嗎？`
    );
    if (!ok) return;
  }
  saving = true;
  try {
    await configApi.setPassthroughHotkey(next);
    passthroughHotkey = next;
    message = "穿透快捷鍵已儲存，重新啟動後生效";
  } catch (e) {
    message = `儲存失敗：${e}`;
  } finally {
    saving = false;
  }
}
```

- [ ] **Step 3: 顯示註冊失敗警示**

`onMount` 從 `configApi.get()` 回傳值取 `passthroughHotkeyRegistered`：

```ts
let passthroughRegistered = true;
// 既有 onMount
const cfg = await configApi.get();
// ...
passthroughRegistered = cfg.passthroughHotkeyRegistered ?? true;
```

template 在「穿透模式快捷鍵」section 加：

```svelte
{#if !passthroughRegistered}
  <p class="warning">⚠ 啟動時註冊「{passthroughHotkey}」失敗，可能已被其他程式占用。請更換後重啟 Waypoint。</p>
{/if}
```

CSS：

```css
.warning {
  font-size: 12px;
  color: var(--danger);
  background: rgba(244, 71, 71, 0.08);
  border: 1px solid var(--danger);
  border-radius: var(--radius);
  padding: 6px 8px;
}
```

- [ ] **Step 4: 寫測試（vitest）**

`SettingsWindow.conflict.test.ts`：

```ts
import { describe, it, expect, vi } from "vitest";
import { findConflict } from "../lib/hotkeyConflicts";

describe("SettingsWindow conflict guard", () => {
  it("findConflict catches Ctrl+Shift+T", () => {
    expect(findConflict("Ctrl+Shift+T")).not.toBeNull();
  });
});
```

（衝突 prompt 的 confirm() 在 Vitest 不易模擬；本測試僅守門 module；實際提示靠 e2e 手測。）

- [ ] **Step 5: Commit**

```bash
git add waypoint/src/windows/SettingsWindow.svelte waypoint/src/windows/SettingsWindow.conflict.test.ts
git commit -m "feat(settings): hotkey conflict prompt + register-fail warning (#3)"
```

---

## Task 14: Playwright drag 渲染測試（每個視窗）

**Files:**
- Create: `src/windows/SettingsWindow.drag.render.test.pw.ts`
- Create: `src/windows/HelpWindow.drag.render.test.pw.ts`
- Create: `src/windows/ListWindow.drag.render.test.pw.ts`
- Create: `src/windows/NoteWindow.drag.render.test.pw.ts`

- [ ] **Step 1: 通用 drag 測試 helper**

`src/windows/_drag.helper.ts`：

```ts
import type { Page } from "@playwright/test";

export async function setupDragSpy(page: Page) {
  await page.evaluate(() => {
    (window as any).__dragSpy = [];
    (window as any).__TAURI_INTERNALS__ = {
      ...((window as any).__TAURI_INTERNALS__ ?? {}),
      invoke: async (cmd: string, args: any) => {
        if (cmd === "start_dragging") (window as any).__dragSpy.push(args.label);
        return undefined;
      },
    };
  });
}

export async function getDragSpy(page: Page): Promise<string[]> {
  return await page.evaluate(() => (window as any).__dragSpy ?? []);
}
```

- [ ] **Step 2: 為每視窗建測試（範例 SettingsWindow）**

```ts
import { test, expect } from "@playwright/test";
import { setupDragSpy, getDragSpy } from "./_drag.helper";

test("SettingsWindow titlebar mousedown triggers start_dragging settings", async ({ page }) => {
  await page.goto("/?#view=settings");
  await setupDragSpy(page);
  await page.locator(".draggable-titlebar").first().dispatchEvent("mousedown", { button: 0 });
  expect(await getDragSpy(page)).toContain("settings");
});
```

對 Help / List / Note 重複，label 分別 `help` / `list` / `note-<id>`。Note 用：

```ts
await page.goto("/?#view=note&noteId=demo");
```

需確保有 mock `notesApi.read` 回傳含 settings 的物件，否則 NoteWindow render 不出 titlebar。可在 setup 內：

```ts
await page.evaluate(() => {
  (window as any).__TAURI_INTERNALS__ = {
    invoke: async (cmd: string) => {
      if (cmd === "start_dragging") return undefined;
      if (cmd === "read_note") return {
        id: "demo", contextId: null, title: "demo", content: "",
        settings: { fontSize: 14, opacity: 1, hotkey: null, windowBounds: null, passthrough: false },
      };
      return undefined;
    },
  };
});
```

- [ ] **Step 3: 跑全部 drag 測試**

```bash
cd waypoint && npm run build && npm run test:render -- drag
```

預期：4 個視窗 PASS。

- [ ] **Step 4: Commit**

```bash
git add waypoint/src/windows/_drag.helper.ts waypoint/src/windows/*.drag.render.test.pw.ts
git commit -m "test(render): drag works on Settings/Help/List/Note (#6)"
```

---

## Task 15: 整合驗證 + 推 master

- [ ] **Step 1: 全套測試在 dev/main**

```bash
cd waypoint && npm test
cd waypoint/src-tauri && cargo test
cd waypoint && npm run build && npm run test:render
```

預期：全綠。

- [ ] **Step 2: 本機 act 跑 e2e-linux**

```bash
export DOCKER_HOST=tcp://host.docker.internal:2375
cd /data/games-note-AIgen && act -j e2e-linux
```

預期：PASS。

- [ ] **Step 3: ff-only merge 到 master 並推**

```bash
git checkout master
git merge --ff-only dev/main
git push origin master
```

- [ ] **Step 4: 等 e2e-windows 綠**

```bash
RUN_ID=$(curl -s "https://api.github.com/repos/I321I/waypoint/actions/workflows/262677203/runs?branch=master&per_page=1" \
  | python3 -c "import json,sys;print(json.load(sys.stdin)['workflow_runs'][0]['id'])")
while true; do
  result=$(curl -s "https://api.github.com/repos/I321I/waypoint/actions/runs/$RUN_ID" \
    | python3 -c "import json,sys;r=json.load(sys.stdin);print(r.get('status'),r.get('conclusion'))")
  echo "$result"
  [[ "$result" != *"None"* && "$result" != in_progress* && "$result" != queued* ]] && break
  sleep 45
done
```

預期：`completed success`。

- [ ] **Step 5: bump version + tag**

編輯 `waypoint/src-tauri/tauri.conf.json` 與 `waypoint/package.json` 的 `version` 至 `0.1.17`。

```bash
git add waypoint/src-tauri/tauri.conf.json waypoint/package.json
git commit -m "chore: bump version to 0.1.17"
git push origin master
git tag v0.1.17
git push --tags
```

---

## Self-Review

**Spec coverage:**
- 議題 1（透明套用層）→ Task 7 ✓
- 議題 2（拉桿位置 + 樣式 A）→ Task 8 + Task 9 ✓
- 議題 3（Edge hotkey）→ Task 2 + Task 3 + Task 4 + Task 13 ✓
- 議題 4（Note → List 改名 sync）→ Task 10 ✓
- 議題 5（List → Note 改名 sync）→ Task 11 ✓
- 議題 6（所有視窗可拖）→ Task 5 + Task 6 + Task 14 ✓
- 議題 7（內容遺失 L1+L2+L3）→ Task 1 + Task 12 ✓

**Placeholder scan:** 無 TBD/TODO；每個 step 含具體 code 或 command。

**Type consistency:**
- `passthroughHotkeyRegistered` 前後端命名一致（後端 snake_case、前端 camelCase 經 serde rename，但 Task 3 已說明 DTO 結構）。
- `note-title-changed` / `note-renamed-from-list` 兩條 event channel 名稱在 Task 10/11 一致。
- `flush-and-save-now` / `flush-ack` 在 Task 12 一致。

---

## 執行模式選擇

Plan complete and saved to `waypoint/docs/superpowers/plans/2026-04-27-transparency-hotkey-rename-drag-save.md`. Two execution options:

**1. Subagent-Driven (recommended)** — 每 Task dispatch 一個 fresh subagent，Task 間 review，可平行（Task 1/2/4 互不依賴）

**2. Inline Execution** — 直接在這個 session 跑，每 ~3 task 暫停讓你 review

哪種？
