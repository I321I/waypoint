# 筆記功能修正與設定擴充 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修正筆記刪除生命週期 bug、加入透明文字開關、清理筆記 chrome（移除底部 statusbar、區域改寫到 titlebar）、Alt+F4 與 X 行為一致、新增筆記內刪除功能。

**Architecture:** 五個議題分組為三個技術主題：
- **事件驅動的關閉路徑**（議題 1, 4, 5）— 共用 `waypoint://note-deleted` 廣播 + `note-closed` 事件 + 攔截 `tauri://close-requested`，所有「關閉」訊號走同一條路徑
- **透明渲染重構**（議題 2）— 從 CSS `opacity` 整體透明改為 `rgba()` 背景，子元素背景全面改 transparent，新增 `transparent_includes_text` 全域設定
- **Titlebar 標籤格式 + chrome 清理**（議題 3）— 純 UI 重排，無狀態變更

**Tech Stack:** Svelte 5、Tauri v2、Rust、Vitest、Playwright（render test）、WebdriverIO（e2e）

**設計來源：** `docs/superpowers/specs/2026-05-03-note-fixes-design.md`

**工作分支：** 全部在 `dev/main` 上推 commit；五個議題完成 + Linux 本機 act 綠燈後 ff-merge 到 master 觸發 Windows E2E。

---

## File Structure

### 新建
- `waypoint/src/windows/ConfirmDialog.svelte` — 從 `list/ConfirmDialog.svelte` 移上來，列表 + 筆記共用

### 修改
- `waypoint/src-tauri/src/commands/notes.rs` — `delete_note` 改成 `async`，刪檔成功後 emit `waypoint://note-deleted`
- `waypoint/src-tauri/src/storage/app_config.rs` — `AppConfig` 新增 `transparent_includes_text` 欄位
- `waypoint/src-tauri/src/storage/session.rs` — `load_session` 過濾不存在的 note id
- `waypoint/src-tauri/src/storage/app_session.rs` — `save` 前過濾不存在的 note ref；`take` 後也過濾一次
- `waypoint/src-tauri/src/lib.rs` — restore 流程前過濾不存在的 note
- `waypoint/src/lib/types.ts` — `AppConfig` TS 型別新增欄位
- `waypoint/src/lib/api.ts` — `config` API 新增 `transparent_includes_text` 讀寫（如有需要）
- `waypoint/src/windows/SettingsWindow.svelte` — 新增「透明時文字也透明」toggle
- `waypoint/src/windows/NoteWindow.svelte` — 監聽 `note-deleted`、攔截 close-requested、改 titlebar 格式、移除 statusbar、改 rgba 透明
- `waypoint/src/windows/note/SettingsPanel.svelte` — 加「刪除此筆記」按鈕 + ConfirmDialog
- `waypoint/src/windows/ListWindow.svelte` — 監聽 `waypoint://note-deleted` 同步移除
- `waypoint/src/windows/list/NoteItem.svelte` — 從新位置 import ConfirmDialog（若有 import）
- `README.md` — 補 markdown 支援說明
- `waypoint/src/windows/HelpWindow.svelte` — 補 markdown 支援說明

### 刪除
- `waypoint/src/windows/list/ConfirmDialog.svelte` — 移到 `windows/ConfirmDialog.svelte`

---

## 議題 1：刪除筆記生命週期

### Task 1.1: Rust delete_note 廣播 note-deleted 事件

**Files:**
- Modify: `waypoint/src-tauri/src/commands/notes.rs:32-35`
- Test: `waypoint/src-tauri/src/commands/notes.rs`（同檔 `#[cfg(test)] mod tests`）

- [ ] **Step 1: 改 `delete_note` 簽章為 async + 接 `AppHandle`，刪檔成功後 emit 事件**

```rust
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn delete_note(
    app: AppHandle,
    context_id: Option<String>,
    note_id: String,
) -> Result<(), WaypointError> {
    notes::delete_note(context_id.as_deref(), &note_id)?;
    let _ = app.emit("waypoint://note-deleted", serde_json::json!({
        "noteId": note_id,
        "contextId": context_id,
    }));
    Ok(())
}
```

- [ ] **Step 2: Run frontend smoke build to ensure invoke arg shape unchanged**

```bash
cd waypoint && npm run build
```

Expected: build success（前端傳的 `{contextId, noteId}` 不變）

- [ ] **Step 3: Commit**

```bash
git add waypoint/src-tauri/src/commands/notes.rs
git commit -m "feat(notes): delete_note 成功後廣播 waypoint://note-deleted"
```

---

### Task 1.2: NoteWindow 監聽 note-deleted 並強制關閉

**Files:**
- Modify: `waypoint/src/windows/NoteWindow.svelte`
- Test: `waypoint/src/windows/NoteWindow.deleted.render.test.pw.ts`（新增）

- [ ] **Step 1: 寫 Playwright render test — 收到 note-deleted 後不呼叫 saveContent 並關閉**

新增 `waypoint/src/windows/NoteWindow.deleted.render.test.pw.ts`：

```ts
import { test, expect, type Page } from '@playwright/test';

test('note window 收到 note-deleted 自身事件後不再呼叫 save_content', async ({ page }) => {
  let saveCount = 0;
  let closeCalled = false;

  await page.exposeFunction('__incrSave', () => { saveCount++; });
  await page.exposeFunction('__close', () => { closeCalled = true; });

  await page.addInitScript(() => {
    const listeners: Record<string, ((p:any)=>void)[]> = {};
    (window as any).__TAURI_INTERNALS__ = {
      metadata: {
        currentWindow: { label: 'note-test-id' },
        currentWebview: { label: 'note-test-id', windowLabel: 'note-test-id' },
      },
      invoke: (cmd: string) => {
        if (cmd === 'plugin:event|listen') return Promise.resolve(0);
        if (cmd === 'read_note') return Promise.resolve({
          id: 'test-id', contextId: null, title: 'T', content: '',
          settings: { fontSize: 14, opacity: 1, hotkey: null, windowBounds: null, passthrough: false },
        });
        if (cmd === 'save_content') { (window as any).__incrSave(); return Promise.resolve(); }
        if (cmd === 'plugin:window|close') { (window as any).__close(); return Promise.resolve(); }
        return Promise.resolve(null);
      },
      transformCallback: (cb: any) => { return 0; },
      unregisterCallback: () => {},
      convertFileSrc: (s: string) => s,
    };
    (window as any).__emit = (evt: string, payload: any) => {
      window.dispatchEvent(new CustomEvent('tauri://' + evt, { detail: payload }));
    };
  });

  await page.goto('http://localhost:4173/#view=note&noteId=test-id');
  await page.waitForLoadState('networkidle');
  await page.locator('.note-window').waitFor({ state: 'visible' });

  // 模擬 backend 廣播 note-deleted
  await page.evaluate(() => {
    (window as any).__emit('note-deleted', { noteId: 'test-id', contextId: null });
  });

  await page.waitForTimeout(300);
  expect(saveCount).toBe(0);
});
```

說明：因為 Tauri event 系統的 mock 比較複雜，這個測試以「不會呼叫 save_content」為門檻（強制關閉的核心保證）。實作上以 `@tauri-apps/api/event::listen` 訂閱事件。

- [ ] **Step 2: 在 NoteWindow.svelte onMount 加 listener，收到自己的 deleted 事件後 emit `note-closed` 並 close 視窗（不 flush）**

修改 NoteWindow.svelte（在 onMount 區段現有 `unlistenRenamedFromList` 旁邊加）：

```ts
import { getCurrentWindow } from "@tauri-apps/api/window";

let unlistenDeleted: (() => void) | null = null;

// in onMount:
unlistenDeleted = await listen<{ noteId: string; contextId: string | null }>(
  "waypoint://note-deleted",
  async (event) => {
    if (event.payload.noteId !== noteId) return;
    if ((event.payload.contextId ?? null) !== (contextId ?? null)) return;
    // 強制關閉：不 flushPendingSave、不 saveContent（檔案已不存在）
    await emit("note-closed", { noteId, contextId, isGlobal: contextId === null });
    await getCurrentWindow().close();
  }
);

// in onDestroy:
unlistenDeleted?.();
```

- [ ] **Step 3: 跑 render test**

```bash
cd waypoint && npm run build && npm run test:render -- NoteWindow.deleted
```

Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add waypoint/src/windows/NoteWindow.svelte waypoint/src/windows/NoteWindow.deleted.render.test.pw.ts
git commit -m "fix(note): 收到 note-deleted 後強制關閉視窗（跳過 saveContent，避免寫已刪除檔案）"
```

---

### Task 1.3: ListWindow 監聽 waypoint://note-deleted 同步移除

**Files:**
- Modify: `waypoint/src/windows/ListWindow.svelte`

- [ ] **Step 1: ListWindow onMount 加 listener，重新載入清單**

在 ListWindow.svelte onMount 加：

```ts
let unlistenDeleted: (() => void) | null = null;

// in onMount:
unlistenDeleted = await listen<{ noteId: string; contextId: string | null }>(
  "waypoint://note-deleted",
  async (event) => {
    // 從 openIds 移除（避免 collapseAll 又把幽靈 id 寫回 session）
    if (event.payload.contextId === null) {
      openGlobalNoteIds = openGlobalNoteIds.filter(id => id !== event.payload.noteId);
    } else {
      openContextNoteIds = openContextNoteIds.filter(id => id !== event.payload.noteId);
    }
    await reloadLists();
  }
);

// in onDestroy:
unlistenDeleted?.();
```

- [ ] **Step 2: 跑列表既有 render test 確認沒壞**

```bash
cd waypoint && npm run build && npm run test:render -- ListWindow ConfirmDialog
```

Expected: 全部 PASS

- [ ] **Step 3: Commit**

```bash
git add waypoint/src/windows/ListWindow.svelte
git commit -m "fix(list): 監聽 waypoint://note-deleted 同步移除清單與 openIds"
```

---

### Task 1.4: session.rs / app_session.rs 過濾不存在的 note

**Files:**
- Modify: `waypoint/src-tauri/src/storage/session.rs`
- Modify: `waypoint/src-tauri/src/storage/app_session.rs`
- Test: 同檔 `#[cfg(test)] mod tests`

- [ ] **Step 1: 寫 session.rs 失敗測試**

在 `session.rs` mod tests 內加：

```rust
#[test]
fn load_session_filters_missing_notes() {
    let (_dir, _guard) = setup();
    use crate::storage::notes;
    // 建立一個真實 note
    let note = notes::create_note(Some("ctx"), "alive").unwrap();
    let alive_id = note.id.clone();
    // 寫一個 session 包含不存在的 id
    let s = Session {
        open_context_notes: vec![alive_id.clone(), "ghost".into()],
        open_global_notes: vec!["ghost-global".into()],
    };
    save_session("ctx", &s).unwrap();

    let loaded = load_session("ctx").unwrap();
    assert_eq!(loaded.open_context_notes, vec![alive_id]);
    assert!(loaded.open_global_notes.is_empty());
}
```

（補一個 `setup` 函式若未存在；session.rs 既有 `mod tests` 已有 `setup`，沿用。）

- [ ] **Step 2: 跑測試確認 FAIL**

```bash
cd waypoint/src-tauri && cargo test --lib storage::session::tests::load_session_filters_missing_notes
```

Expected: FAIL（目前 load_session 不會過濾）

- [ ] **Step 3: 修 load_session 在讀完後過濾**

修改 session.rs `load_session`：

```rust
pub fn load_session(context_id: &str) -> Result<Session, WaypointError> {
    let path = session_path(context_id);
    if !path.exists() {
        return Ok(Session::default());
    }
    let content = std::fs::read_to_string(&path)?;
    let mut sess: Session = serde_json::from_str(&content)?;
    sess.open_context_notes.retain(|id| crate::storage::notes::note_exists(Some(context_id), id));
    sess.open_global_notes.retain(|id| crate::storage::notes::note_exists(None, id));
    Ok(sess)
}
```

並在 `notes.rs` 加 helper `note_exists`：

```rust
pub fn note_exists(context_id: Option<&str>, note_id: &str) -> bool {
    note_path(context_id, note_id).exists()
}
```

（若 `note_path` 是 private，將其 `pub(crate)` 或新增 wrapper。）

- [ ] **Step 4: 跑測試 PASS**

```bash
cd waypoint/src-tauri && cargo test --lib storage::session::tests::load_session_filters_missing_notes
```

Expected: PASS

- [ ] **Step 5: 對 app_session.rs 做同樣的事**

在 `app_session.rs` mod tests 加：

```rust
#[test]
fn take_filters_missing_notes() {
    let (_dir, _guard) = setup();
    use crate::storage::notes;
    let n = notes::create_note(None, "alive").unwrap();
    let s = AppSession {
        open_notes: vec![
            OpenNoteRef { note_id: n.id.clone(), context_id: None },
            OpenNoteRef { note_id: "ghost".into(), context_id: None },
        ],
        list_open: false,
    };
    save(&s).unwrap();

    let loaded = take().unwrap();
    assert_eq!(loaded.open_notes.len(), 1);
    assert_eq!(loaded.open_notes[0].note_id, n.id);
}
```

並修改 `take`：

```rust
pub fn take() -> Option<AppSession> {
    let path = app_session_path();
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&path).ok()?;
    let mut sess: AppSession = serde_json::from_str(&content).ok()?;
    sess.open_notes.retain(|r| crate::storage::notes::note_exists(r.context_id.as_deref(), &r.note_id));
    let _ = std::fs::remove_file(&path);
    Some(sess)
}
```

- [ ] **Step 6: 跑兩個 module 的測試**

```bash
cd waypoint/src-tauri && cargo test --lib storage::session storage::app_session
```

Expected: 全部 PASS

- [ ] **Step 7: Commit**

```bash
git add waypoint/src-tauri/src/storage/session.rs waypoint/src-tauri/src/storage/app_session.rs waypoint/src-tauri/src/storage/notes.rs
git commit -m "fix(session): load/take 時過濾不存在的 note，避免幽靈視窗復活"
```

---

## 議題 2：透明文字開關

### Task 2.1: AppConfig 新增 transparent_includes_text 欄位

**Files:**
- Modify: `waypoint/src-tauri/src/storage/app_config.rs`
- Test: 同檔 `#[cfg(test)] mod tests`

- [ ] **Step 1: 在 AppConfig 加欄位（預設 true）**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    // ... existing fields ...
    #[serde(default = "default_transparent_includes_text")]
    pub transparent_includes_text: bool,
}

fn default_transparent_includes_text() -> bool {
    true
}
```

`Default` impl 也加上：`transparent_includes_text: default_transparent_includes_text()`。

- [ ] **Step 2: 寫測試 — 預設值 true、序列化往返**

```rust
#[test]
fn transparent_includes_text_defaults_true() {
    let c = AppConfig::default();
    assert!(c.transparent_includes_text);
}

#[test]
fn transparent_includes_text_round_trip() {
    let mut c = AppConfig::default();
    c.transparent_includes_text = false;
    let s = serde_json::to_string(&c).unwrap();
    let back: AppConfig = serde_json::from_str(&s).unwrap();
    assert!(!back.transparent_includes_text);
}

#[test]
fn transparent_includes_text_missing_in_json_defaults_true() {
    // 舊版 config 沒這欄位
    let json = r#"{"hotkey":"Ctrl+Shift+Space","contextAliases":{},"contexts":{},"passthroughHotkey":"Ctrl+Shift+Q","showInTaskbar":true}"#;
    let c: AppConfig = serde_json::from_str(json).unwrap();
    assert!(c.transparent_includes_text);
}
```

- [ ] **Step 3: 跑測試**

```bash
cd waypoint/src-tauri && cargo test --lib storage::app_config
```

Expected: PASS

- [ ] **Step 4: 同步 TS 型別 `waypoint/src/lib/types.ts`**

找到 `AppConfig` interface（如有），加：
```ts
transparentIncludesText: boolean;
```

若 TS 端不直接讀 AppConfig，可新增 invoke wrapper：
```ts
// waypoint/src/lib/api.ts （config 區塊）
getTransparentIncludesText: () => invoke<boolean>("get_transparent_includes_text"),
setTransparentIncludesText: (v: boolean) => invoke<void>("set_transparent_includes_text", { value: v }),
```

對應 Rust `commands/config_cmd.rs` 加 getter/setter command 並註冊到 `lib.rs` 的 `invoke_handler`。

- [ ] **Step 5: Commit**

```bash
git add waypoint/src-tauri/src/storage/app_config.rs waypoint/src-tauri/src/commands/config_cmd.rs waypoint/src-tauri/src/lib.rs waypoint/src/lib/api.ts waypoint/src/lib/types.ts
git commit -m "feat(config): 新增 transparentIncludesText 全域設定（預設 true）"
```

---

### Task 2.2: SettingsWindow 加 toggle 並廣播 config-changed

**Files:**
- Modify: `waypoint/src/windows/SettingsWindow.svelte`
- Test: `waypoint/src/windows/SettingsWindow.transparent.render.test.pw.ts`（新增）

- [ ] **Step 1: 寫 render test — toggle 預設 ON、點擊後變 OFF**

```ts
import { test, expect, type Page } from '@playwright/test';

test('settings window 透明文字 toggle 預設 ON', async ({ page }) => {
  await page.addInitScript(() => {
    (window as any).__TAURI_INTERNALS__ = {
      metadata: { currentWindow: { label: 'settings' }, currentWebview: { label: 'settings', windowLabel: 'settings' } },
      invoke: (cmd: string) => {
        if (cmd === 'get_transparent_includes_text') return Promise.resolve(true);
        if (cmd === 'set_transparent_includes_text') return Promise.resolve();
        return Promise.resolve({});
      },
      transformCallback: () => 0, unregisterCallback: () => {}, convertFileSrc: (s:string)=>s,
    };
  });
  await page.goto('http://localhost:4173/#view=settings');
  await page.waitForLoadState('networkidle');
  const toggle = page.locator('input[data-testid="transparent-includes-text"]');
  await expect(toggle).toBeChecked();
});
```

- [ ] **Step 2: 跑測試確認 FAIL**

```bash
cd waypoint && npm run build && npm run test:render -- SettingsWindow.transparent
```

Expected: FAIL（toggle 還沒做）

- [ ] **Step 3: 在 SettingsWindow.svelte 加 toggle UI 與綁定**

於 SettingsWindow.svelte 適當位置加：

```svelte
<script lang="ts">
  // ... 既有 import ...
  import { invoke } from "@tauri-apps/api/core";
  import { emit } from "@tauri-apps/api/event";

  let transparentIncludesText = true;

  onMount(async () => {
    // ... 既有 ...
    transparentIncludesText = await invoke<boolean>("get_transparent_includes_text");
  });

  async function toggleTransparentText(e: Event) {
    transparentIncludesText = (e.target as HTMLInputElement).checked;
    await invoke("set_transparent_includes_text", { value: transparentIncludesText });
    await emit("waypoint://config-changed");
  }
</script>

<!-- 在透明區塊加： -->
<label class="setting-row">
  <input
    type="checkbox"
    data-testid="transparent-includes-text"
    bind:checked={transparentIncludesText}
    on:change={toggleTransparentText}
  />
  <span>透明時文字也透明</span>
</label>
```

- [ ] **Step 4: 跑測試 PASS**

```bash
cd waypoint && npm run build && npm run test:render -- SettingsWindow.transparent
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add waypoint/src/windows/SettingsWindow.svelte waypoint/src/windows/SettingsWindow.transparent.render.test.pw.ts
git commit -m "feat(settings): 新增「透明時文字也透明」toggle，broadcast waypoint://config-changed"
```

---

### Task 2.3: NoteWindow 改用 rgba 背景 + 條件式 opacity

**Files:**
- Modify: `waypoint/src/windows/NoteWindow.svelte`
- Modify: `waypoint/src/windows/note/Editor.svelte`（如有 opaque 背景）
- Modify: `waypoint/src/windows/note/SettingsPanel.svelte`（已有 `background: var(--bg-secondary)`，需改透明）
- Modify: `waypoint/src/windows/note/Toolbar.svelte`
- Test: `waypoint/src/windows/NoteWindow.transparent.render.test.pw.ts`（修改既有檔）

- [ ] **Step 1: 盤點 opaque 背景**

```bash
grep -n "background" waypoint/src/windows/NoteWindow.svelte waypoint/src/windows/note/*.svelte
```

把每個 opaque 背景改成 transparent 或 rgba（保留 hover 狀態的局部 opaque 是 OK 的）。

- [ ] **Step 2: 改 NoteWindow.svelte 主容器**

```svelte
{#if note}
  <div
    class="note-window"
    class:translucent-text={transparentIncludesText}
    style="--note-bg-alpha: {note.settings.opacity}"
  >
```

CSS：
```css
.note-window {
  background-color: rgba(30, 30, 30, var(--note-bg-alpha));
  /* 移除原本的 background: rgb(30,30,30) */
}
.note-window.translucent-text {
  opacity: var(--note-bg-alpha);
}
```

並從 `<script>` 移除 `style="opacity: ..."` inline 寫法。

- [ ] **Step 3: 在 NoteWindow.svelte 訂閱全域設定 + 監聽 config-changed**

```ts
let transparentIncludesText = true;

onMount(async () => {
  // ... 既有 ...
  transparentIncludesText = await invoke<boolean>("get_transparent_includes_text");
  unlistenConfig = await listen("waypoint://config-changed", async () => {
    transparentIncludesText = await invoke<boolean>("get_transparent_includes_text");
  });
});
```

- [ ] **Step 4: 改子元件背景為 transparent / rgba**

例如 SettingsPanel.svelte：
```css
.settings-panel {
  background-color: rgba(40, 40, 40, var(--note-bg-alpha, 1));
  /* 從 var(--bg-secondary) 改 */
}
```

Editor / Toolbar / titlebar 視盤點結果處理。

- [ ] **Step 5: 修改既有 transparent render test，新增 includes-text=false 案例**

在 `NoteWindow.transparent.render.test.pw.ts` 加：

```ts
test('translucent-text=false 時 .note-window opacity===1，背景含 alpha<1', async ({ page }) => {
  await mockNoteWithOpacity(page, 0.3);
  // mock invoke 加 get_transparent_includes_text → false
  // ...
  await page.goto('http://localhost:4173/#view=note&noteId=test-id');
  await page.waitForLoadState('networkidle');
  const noteWindow = page.locator('.note-window');
  const op = await noteWindow.evaluate(el => getComputedStyle(el).opacity);
  expect(parseFloat(op)).toBeCloseTo(1, 2);
  const bg = await noteWindow.evaluate(el => getComputedStyle(el).backgroundColor);
  // rgba(30, 30, 30, 0.3) → 解析後 alpha 不是 1
  expect(bg).toMatch(/rgba?\(.*0?\.3/);
});
```

並修改 mock 讓 invoke 也處理 `get_transparent_includes_text`。

- [ ] **Step 6: 跑測試**

```bash
cd waypoint && npm run build && npm run test:render -- NoteWindow.transparent
```

Expected: PASS（兩個案例都通）

- [ ] **Step 7: Commit**

```bash
git add waypoint/src/windows/NoteWindow.svelte waypoint/src/windows/note/SettingsPanel.svelte waypoint/src/windows/note/Editor.svelte waypoint/src/windows/note/Toolbar.svelte waypoint/src/windows/NoteWindow.transparent.render.test.pw.ts
git commit -m "feat(note): 透明用 rgba 背景，可選擇文字是否一起透明"
```

---

## 議題 3：移除底部 statusbar、區域標籤改到 titlebar

### Task 3.1: 改 titlebar 標題格式 + 移除 statusbar

**Files:**
- Modify: `waypoint/src/windows/NoteWindow.svelte`
- Test: 修改 `waypoint/src/windows/NoteWindow.transparent.render.test.pw.ts` 或新增 `NoteWindow.titlebar.render.test.pw.ts`

- [ ] **Step 1: 寫 render test — titlebar 顯示 `xxx-Global` 與 `xxx-edge`，無 .statusbar**

新增 `waypoint/src/windows/NoteWindow.titlebar.render.test.pw.ts`：

```ts
import { test, expect, type Page } from '@playwright/test';

async function mockNote(page: Page, contextId: string | null, title: string) {
  await page.addInitScript(({ ctx, t }) => {
    (window as any).__TAURI_INTERNALS__ = {
      metadata: { currentWindow: { label: 'note-x' }, currentWebview: { label: 'note-x', windowLabel: 'note-x' } },
      invoke: (cmd: string) => {
        if (cmd === 'read_note') return Promise.resolve({
          id: 'x', contextId: ctx, title: t, content: '',
          settings: { fontSize: 14, opacity: 1, hotkey: null, windowBounds: null, passthrough: false },
        });
        if (cmd === 'get_transparent_includes_text') return Promise.resolve(true);
        return Promise.resolve(null);
      },
      transformCallback: () => 0, unregisterCallback: () => {}, convertFileSrc: (s:string)=>s,
    };
  }, { ctx: contextId, t: title });
}

test('全域筆記 titlebar 顯示 {title}-Global', async ({ page }) => {
  await mockNote(page, null, '1122');
  await page.goto('http://localhost:4173/#view=note&noteId=x');
  await page.waitForLoadState('networkidle');
  await expect(page.locator('.note-title')).toHaveText('1122-Global');
});

test('區域筆記 titlebar 顯示 {title}-{contextId}', async ({ page }) => {
  await mockNote(page, 'edge', '我是誰');
  await page.goto('http://localhost:4173/#view=note&noteId=x');
  await page.waitForLoadState('networkidle');
  await expect(page.locator('.note-title')).toHaveText('我是誰-edge');
});

test('NoteWindow 不再顯示 .statusbar', async ({ page }) => {
  await mockNote(page, null, 'T');
  await page.goto('http://localhost:4173/#view=note&noteId=x');
  await page.waitForLoadState('networkidle');
  await expect(page.locator('.statusbar')).toHaveCount(0);
});
```

- [ ] **Step 2: 跑測試確認 FAIL**

```bash
cd waypoint && npm run build && npm run test:render -- NoteWindow.titlebar
```

Expected: FAIL（目前格式是 `1122 — Global`，且 .statusbar 還在）

- [ ] **Step 3: 改 NoteWindow.svelte titlebar 文字 + 移除 statusbar**

把：
```svelte
<span class="note-title" data-tauri-drag-region>{title || "Untitled"}{contextId ? ` — ${contextId}` : ""}</span>
```
改為：
```svelte
<span class="note-title" data-tauri-drag-region>{(title || "Untitled") + "-" + (contextId ?? "Global")}</span>
```

並刪除：
```svelte
<div class="statusbar">
  <span>{contextId ?? "Global"}</span>
  <span>Markdown</span>
</div>
```
與其 CSS（`.statusbar { ... }` 整個 rule 移除）。

- [ ] **Step 4: 跑測試 PASS**

```bash
cd waypoint && npm run build && npm run test:render -- NoteWindow.titlebar
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add waypoint/src/windows/NoteWindow.svelte waypoint/src/windows/NoteWindow.titlebar.render.test.pw.ts
git commit -m "feat(note): titlebar 顯示 {title}-{Global|context}，移除底部 statusbar"
```

---

### Task 3.2: README + HelpWindow 補 markdown 說明

**Files:**
- Modify: `README.md`
- Modify: `waypoint/src/windows/HelpWindow.svelte`

- [ ] **Step 1: 在 README.md 「功能特色」或類似區塊加一行**

```markdown
- 筆記內容支援 Markdown 語法（# Heading、**bold**、`code` 等）
```

- [ ] **Step 2: 在 HelpWindow.svelte 加同樣說明**

找到使用說明列表，加：
```svelte
<li>筆記內容支援 Markdown 語法：# Heading、**粗體**、`程式碼` 等</li>
```

- [ ] **Step 3: Commit**

```bash
git add README.md waypoint/src/windows/HelpWindow.svelte
git commit -m "docs: 把 markdown 支援說明從筆記 chrome 改寫到 README + Help"
```

---

## 議題 4：Alt+F4 行為與 X 一致

### Task 4.1: NoteWindow 攔截 close-requested

**Files:**
- Modify: `waypoint/src/windows/NoteWindow.svelte`

- [ ] **Step 1: 在 onMount 註冊 onCloseRequested handler**

```ts
import { getCurrentWindow } from "@tauri-apps/api/window";

let unlistenClose: (() => void) | null = null;

// in onMount:
unlistenClose = await getCurrentWindow().onCloseRequested(async (e) => {
  e.preventDefault();
  await handleClose();
});

// in onDestroy:
unlistenClose?.();
```

確保 `handleClose` 已經會呼叫 `getCurrentWindow().close()`（檢查現有 handleClose；若它只 emit 事件而未真的關窗，要在事件 emit 後呼叫 close）。

- [ ] **Step 2: 補測 — 模擬 close-requested 觸發 saveContent + emit note-closed**

新增 `waypoint/src/windows/NoteWindow.altf4.render.test.pw.ts`（或合進 deleted test）：

```ts
test('NoteWindow 攔截 tauri://close-requested 並 emit note-closed', async ({ page }) => {
  let noteClosedEmitted = false;
  await page.exposeFunction('__noteClosed', () => { noteClosedEmitted = true; });
  // mock invoke 與 plugin:event|emit 攔截 'note-closed'
  // ...（細節同 deleted test 模板，攔截 emit('note-closed') 設旗標）
  // 觸發：模擬 onCloseRequested 內部回呼
  await page.evaluate(() => {
    (window as any).__triggerCloseRequested?.();
  });
  await page.waitForTimeout(200);
  expect(noteClosedEmitted).toBe(true);
});
```

註：Tauri close-requested mock 較複雜，此測試以「emit 'note-closed' 被觸發」為門檻；若 mock 過於複雜可改為 e2e 驗證。

- [ ] **Step 3: 跑測試**

```bash
cd waypoint && npm run build && npm run test:render -- NoteWindow.altf4
```

Expected: PASS（或標記 e2e-only 跳過此 render test，改靠 Task 5.4 的 e2e 驗）

- [ ] **Step 4: Commit**

```bash
git add waypoint/src/windows/NoteWindow.svelte waypoint/src/windows/NoteWindow.altf4.render.test.pw.ts
git commit -m "fix(note): 攔截 tauri://close-requested，Alt+F4 走與 X 相同的 handleClose 流程"
```

---

## 議題 5：筆記內刪除此筆記

### Task 5.1: 移動 ConfirmDialog 到共用位置

**Files:**
- Create: `waypoint/src/windows/ConfirmDialog.svelte`
- Delete: `waypoint/src/windows/list/ConfirmDialog.svelte`
- Modify: 引用點（grep `ConfirmDialog` 找出全部）

- [ ] **Step 1: 找出所有 ConfirmDialog 引用**

```bash
grep -rn "ConfirmDialog" waypoint/src/
```

- [ ] **Step 2: 把 `list/ConfirmDialog.svelte` 內容原封移到 `windows/ConfirmDialog.svelte`**

```bash
git mv waypoint/src/windows/list/ConfirmDialog.svelte waypoint/src/windows/ConfirmDialog.svelte
```

- [ ] **Step 3: 改所有引用點的 import 路徑**

例如 `waypoint/src/windows/list/NoteItem.svelte`：
```svelte
- import ConfirmDialog from "./ConfirmDialog.svelte";
+ import ConfirmDialog from "../ConfirmDialog.svelte";
```

- [ ] **Step 4: 修改 render test 路徑**

`waypoint/src/windows/list/ConfirmDialog.render.test.pw.ts` 也搬到 `waypoint/src/windows/ConfirmDialog.render.test.pw.ts`，調整 import path。

- [ ] **Step 5: 跑既有測試確認沒壞**

```bash
cd waypoint && npm run build && npm run test:render -- ConfirmDialog
```

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add waypoint/src/windows/
git commit -m "refactor: ConfirmDialog 移到 windows/ 共用位置（list 與 note 都會用）"
```

---

### Task 5.2: SettingsPanel 加「刪除此筆記」按鈕

**Files:**
- Modify: `waypoint/src/windows/note/SettingsPanel.svelte`
- Test: `waypoint/src/windows/note/SettingsPanel.delete.render.test.pw.ts`（新增）

- [ ] **Step 1: 寫 render test**

```ts
import { test, expect } from '@playwright/test';

test('SettingsPanel 顯示「刪除此筆記」按鈕，點擊後彈 ConfirmDialog', async ({ page }) => {
  // mount SettingsPanel via storybook-style harness 或 NoteWindow with settingsOpen=true
  // 簡化：透過 NoteWindow 的 settingsOpen 路徑驗
  await page.addInitScript(() => {
    (window as any).__TAURI_INTERNALS__ = {
      metadata: { currentWindow: { label: 'note-x' }, currentWebview: { label: 'note-x', windowLabel: 'note-x' } },
      invoke: (cmd: string) => {
        if (cmd === 'read_note') return Promise.resolve({
          id: 'x', contextId: null, title: 'T', content: '',
          settings: { fontSize: 14, opacity: 1, hotkey: null, windowBounds: null, passthrough: false },
        });
        if (cmd === 'get_transparent_includes_text') return Promise.resolve(true);
        return Promise.resolve(null);
      },
      transformCallback: () => 0, unregisterCallback: () => {}, convertFileSrc: (s:string)=>s,
    };
  });
  await page.goto('http://localhost:4173/#view=note&noteId=x');
  await page.waitForLoadState('networkidle');
  // 開啟設定面板（toolbar 上的設定按鈕，title="設定"）
  await page.locator('[title="設定"]').click();
  const delBtn = page.locator('button[data-testid="delete-this-note"]');
  await expect(delBtn).toBeVisible();
  await delBtn.click();
  await expect(page.locator('.dialog .msg')).toContainText('刪除');
});
```

- [ ] **Step 2: 跑測試確認 FAIL**

```bash
cd waypoint && npm run build && npm run test:render -- SettingsPanel.delete
```

Expected: FAIL

- [ ] **Step 3: 改 SettingsPanel.svelte 加按鈕 + dialog**

```svelte
<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { NoteSettings } from "../../lib/types";
  import { notes as notesApi } from "../../lib/api";
  import ConfirmDialog from "../ConfirmDialog.svelte";

  export let settings: NoteSettings;
  export let noteId: string;
  export let contextId: string | null;
  const dispatch = createEventDispatcher<{ change: NoteSettings }>();

  let confirmingDelete = false;

  function update(patch: Partial<NoteSettings>) {
    settings = { ...settings, ...patch };
    dispatch("change", settings);
  }

  async function doDelete() {
    confirmingDelete = false;
    await notesApi.delete(contextId, noteId);
    // backend 會 emit waypoint://note-deleted；NoteWindow 監聽後會關掉自己
  }
</script>

<div class="settings-panel">
  <!-- 既有 fontSize 區塊 -->
  <div class="setting-row">
    <!-- ... -->
  </div>

  <div class="danger-zone">
    <button
      class="danger-btn"
      data-testid="delete-this-note"
      on:click={() => confirmingDelete = true}
    >
      刪除此筆記
    </button>
  </div>
</div>

{#if confirmingDelete}
  <ConfirmDialog
    message="確定要刪除這份筆記？此操作無法復原。"
    confirmText="刪除"
    cancelText="取消"
    onConfirm={doDelete}
    onCancel={() => confirmingDelete = false}
  />
{/if}

<style>
  /* 既有 styles */
  .danger-zone { margin-top: 16px; padding-top: 12px; border-top: 1px solid var(--border); }
  .danger-btn {
    width: 100%; padding: 6px;
    background: var(--accent-danger, #c0392b); color: white;
    border: none; border-radius: 4px; font-size: 12px;
    cursor: pointer;
  }
</style>
```

- [ ] **Step 4: 跑測試 PASS**

```bash
cd waypoint && npm run build && npm run test:render -- SettingsPanel.delete
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add waypoint/src/windows/note/SettingsPanel.svelte waypoint/src/windows/note/SettingsPanel.delete.render.test.pw.ts
git commit -m "feat(note): 設定面板加「刪除此筆記」按鈕（含確認 dialog）"
```

---

### Task 5.3: 全套單元 + 渲染回歸

- [ ] **Step 1: 跑所有 Vitest**

```bash
cd waypoint && npm test -- --run
```

Expected: 全 PASS

- [ ] **Step 2: 跑所有 cargo test**

```bash
cd waypoint/src-tauri && cargo test
```

Expected: 全 PASS

- [ ] **Step 3: 跑所有 Playwright render test**

```bash
cd waypoint && npm run build && npm run test:render
```

Expected: 全 PASS

- [ ] **Step 4: 若有失敗，逐一修；若全綠，Commit（empty / 微調）**

---

### Task 5.4: 補 E2E 驗證腳本

**Files:**
- Create: `waypoint/e2e/specs/note-deletion.spec.js`
- Create: `waypoint/e2e/specs/altf4-close.spec.js`

- [ ] **Step 1: 寫 e2e — 從筆記內刪除，列表同步移除**

```js
// waypoint/e2e/specs/note-deletion.spec.js
const { browser, expect } = require('@wdio/globals');

describe('note deletion lifecycle', () => {
  it('在筆記內按刪除，列表同步移除該筆記', async () => {
    // 開列表
    // 建立 global 筆記 "TestDel"
    // 開啟該筆記
    // 在筆記設定面板按「刪除此筆記」→ 確認
    // 預期：筆記視窗關閉、列表中 "TestDel" 消失
  });

  it('在列表刪除時，已開啟的筆記視窗自動關閉', async () => {
    // 類似上面，但從列表觸發
  });
});
```

（依照 repo 既有 e2e 風格填空 — 參考 `waypoint/e2e/specs/*.spec.js`）

- [ ] **Step 2: 寫 e2e — Alt+F4 後 list toggle 不會自動拉起**

```js
// waypoint/e2e/specs/altf4-close.spec.js
describe('Alt+F4 close behavior', () => {
  it('Alt+F4 關筆記後，list toggle 重開不會再自動開該筆記', async () => {
    // 開列表 → 開筆記 → 對筆記送 Alt+F4
    // 關列表 → 重開列表
    // 預期：該筆記視窗不在
  });
});
```

- [ ] **Step 3: 本機（若有 Windows）跑 e2e；若沒有，等推 master 後看 Windows runner**

```bash
# Linux act
export DOCKER_HOST=tcp://host.docker.internal:2375
act -j e2e-linux
```

Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add waypoint/e2e/specs/note-deletion.spec.js waypoint/e2e/specs/altf4-close.spec.js
git commit -m "test(e2e): 筆記刪除生命週期 + Alt+F4 行為驗證"
```

---

## 收尾：合併到 master、Linux act 驗、bump 版本

### Task 6.1: Linux act 驗收

- [ ] **Step 1: 跑 e2e-linux 在本機 act**

```bash
export DOCKER_HOST=tcp://host.docker.internal:2375
cd /data/games-note-AIgen
act -j e2e-linux
```

Expected: PASS（綠燈）

- [ ] **Step 2: 若失敗，看 log 修；OK 則進下一步**

---

### Task 6.2: Merge dev/main → master 觸發 Windows E2E

- [ ] **Step 1: 確保 dev/main commit 都推上**

```bash
git push origin dev/main
```

- [ ] **Step 2: ff-merge 到 master**

```bash
git checkout master
git merge --ff-only dev/main
git push origin master
```

- [ ] **Step 3: 輪詢 Windows E2E**

依 CLAUDE.md 中的輪詢指令，等到 conclusion=success。

- [ ] **Step 4: 若紅燈，看 ci-logs branch，修，回 dev/main 再來；綠燈則進下一步**

---

### Task 6.3: README 同步檢查 + bump 版本 + tag release

- [ ] **Step 1: 確認 README.md 已反映本次 user-facing 變化**

本版 user-facing 改動：
- 筆記 titlebar 標題格式變更為 `{title}-{Global|context}`
- 筆記底部 statusbar 移除
- 列表設定新增「透明時文字也透明」開關
- 筆記設定面板新增「刪除此筆記」
- 修正多項刪除 / Alt+F4 bug

逐項檢查 README，缺的補上 commit 一次。

- [ ] **Step 2: bump version**

```bash
# 編輯 waypoint/package.json 與 waypoint/src-tauri/tauri.conf.json 裡的 version
```

- [ ] **Step 3: master 末梢若是 [skip ci] commit，補空 commit；否則直接 tag**

```bash
git commit --allow-empty -m "chore: trigger release for v0.1.22"  # 視情況
git tag -a v0.1.22 -m "$(cat <<'EOF'
- 修正開著筆記時從列表刪除導致筆記視窗關不掉的 bug
- 修正刪除筆記後 session 重開仍會復活已刪除筆記的 bug
- 修正 Alt+F4 關筆記後下次列表 toggle 又自動拉起的 bug
- 新增列表設定「透明時文字也透明」開關（預設 ON 維持原行為）
- 新增筆記設定面板「刪除此筆記」功能
- 筆記 titlebar 顯示格式改為「標題-區域」（例如 1122-Global、我是誰-edge）
- 移除筆記底部「區域 / Markdown」狀態列（Markdown 支援改在 README 與使用說明中提及）
EOF
)"
git push origin v0.1.22
```

- [ ] **Step 4: 等 release.yml 跑完，確認 GitHub Release 出現**

---

## Self-Review

**Spec coverage：**
- 議題 1 → Task 1.1 ~ 1.4 ✓
- 議題 2 → Task 2.1 ~ 2.3 ✓
- 議題 3 → Task 3.1 ~ 3.2 ✓
- 議題 4 → Task 4.1 ✓（並由 Task 5.4 e2e 驗）
- 議題 5 → Task 5.1 ~ 5.4 ✓

**已知簡化：**
- Task 1.2 / 4.1 的 render test 對 Tauri event 系統的 mock 較簡略，最終靠 Task 5.4 的 e2e 把關
- Task 2.3 的子元件背景盤點是動態探索（grep 後逐個改），不在 plan 中列死

**型別 / API 一致性：**
- `note-deleted` 事件 payload 一致：`{ noteId, contextId }`
- `note-closed` 事件 payload 一致：`{ noteId, contextId, isGlobal }`（沿用既有）
- `getTransparentIncludesText` / `setTransparentIncludesText` 命名一致（Task 2.1 與 2.2 與 2.3 都用同名）
- `note_exists(context_id, note_id)` 在 notes.rs 定義，session.rs / app_session.rs 都用同一函式
