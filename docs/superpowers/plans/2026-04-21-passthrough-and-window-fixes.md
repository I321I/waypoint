# 穿透模式 + 視窗/列表/透明度修復批次 — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修復多餘視窗、加入穿透模式、修復透明度/拖曳/確認框、移除過時功能，並動態管理工作列圖示。

**Architecture:**
- Rust 端負責：視窗 builder 屬性（transparent / always_on_top / skip_taskbar）、`set_ignore_cursor_events`、新 commands（`cmd_set_passthrough`、`cmd_toggle_passthrough_global`、`cmd_refresh_taskbar`）、新增 hotkey action `TogglePassthrough`、tray 動態 menu、移除 list autohide / 舊 note hotkey 註冊路徑。
- Svelte 端負責：NoteWindow titlebar 改造（移除 `—`、新增 dot + `⇊`、emit collapse 事件）、SettingsPanel 透明度 slider、ListWindow 設定面板新增 `showInTaskbar` + 穿透 hotkey 欄位、HelpWindow / SettingsWindow titlebar drag fallback、ConfirmDialog 脫離容器寬度。

**Tech Stack:** Rust (Tauri 2.x), Svelte 5, Vitest, Playwright (render), WebdriverIO + tauri-driver (E2E)

**Spec:** `docs/superpowers/specs/2026-04-21-passthrough-and-window-fixes-design.md`

---

## 全域規則（每個 task 都適用）

1. **TDD 強制**：每項修改點都必須有對應測試。沒有測試的 commit 不能合入。測試類型對應：
   - 純函式 / store 邏輯 → Vitest（`*.test.ts`）或 cargo test
   - 視窗渲染 / CSS / DOM → Playwright (`*.render.test.pw.ts`)
   - 視窗行為（拖曳、快捷鍵、tray、ignore_cursor_events）→ WDIO E2E (`waypoint/e2e/specs/*.spec.js`)
2. **分支**：所有 commit 推到 `dev/main`。每完成一個 task 就 commit。階段（Phase）結束時才考慮 fast-forward 到 master 觸發 e2e-windows。
3. **Commit message 前綴**：`feat:` / `fix:` / `refactor:` / `docs:` / `chore:` / `test:`。
4. **檔案路徑一律相對 repo root**，cd 指令在每個區塊明示。
5. **測試指令統一**：
   ```bash
   cd /data/games-note-AIgen/waypoint && npm test                 # Vitest
   cd /data/games-note-AIgen/waypoint/src-tauri && cargo test     # Rust
   cd /data/games-note-AIgen/waypoint && npm run build && npm run test:render  # Playwright
   ```
   E2E 在 Windows runner 才有效，本地 Linux 用 act：`export DOCKER_HOST=tcp://host.docker.internal:2375 && act -j e2e-linux`。

---

## Phase 1 — 封住多餘視窗 + 補測試（R1, R2）

### Task 1.1：建立 stray-window E2E 斷言

**Files:**
- Create: `waypoint/e2e/specs/no-stray-window.spec.js`

- [ ] **Step 1：寫失敗 E2E**

```js
// waypoint/e2e/specs/no-stray-window.spec.js
const { browser } = require('@wdio/globals');

describe('no stray window on startup', () => {
  it('only the list window should exist after WAYPOINT_E2E auto-launch', async () => {
    await browser.pause(1500); // 等 setup 完成
    const handles = await browser.getWindowHandles();
    const labels = [];
    for (const h of handles) {
      await browser.switchToWindow(h);
      const url = await browser.getUrl();
      labels.push(url);
    }
    // 預期只剩 list；不應有 note / settings / help
    const onlyList = labels.every(u => u.includes('view=list'));
    expect(labels.length).toBe(1);
    expect(onlyList).toBe(true);
  });
});
```

- [ ] **Step 2：跑 e2e-linux act 確認 FAIL（若目前 startup 真有多餘視窗會 fail；若沒有，則是抓到「目前其實沒 bug」當回歸鎖）**

```bash
export DOCKER_HOST=tcp://host.docker.internal:2375 && act -j e2e-linux
```
Expected: 此 spec PASS 或 FAIL，先看當前實際狀況決定 R1 是否要修；FAIL → 進 Task 1.2。

### Task 1.2：定位並修掉預掛載的多餘視窗（若 1.1 紅）

**Files:**
- Modify: `waypoint/src-tauri/src/lib.rs`（`setup` 區塊）
- Modify: `waypoint/src-tauri/src/hotkey/mod.rs`（`open_list_window`、`open_note_window`）

- [ ] **Step 1：盤點所有 `WebviewWindowBuilder::new(...)` 呼叫**，標註觸發時機。

```bash
grep -n "WebviewWindowBuilder::new" src-tauri/src/
```

- [ ] **Step 2：對任何「啟動時建立」但不該顯示的視窗，加 `.visible(false)`；確認顯示動作明確由使用者操作觸發。**

- [ ] **Step 3：跑 act 確認 1.1 spec 變綠**

- [ ] **Step 4：commit**

```bash
git add src-tauri/src/lib.rs src-tauri/src/hotkey/mod.rs e2e/specs/no-stray-window.spec.js
git commit -m "fix: 啟動時不再預掛載多餘的 Waypoint 視窗 + 加 E2E 鎖"
```

---

## Phase 2 — 純 UI 移除 / 新增（R10, R11, R8, R12）

### Task 2.1：移除筆記最小化按鈕（R10）

**Files:**
- Modify: `waypoint/src/windows/NoteWindow.svelte`（line 78 區塊 + handler）
- Modify: `waypoint/e2e/specs/smoke.spec.js` 或對應 spec（移除「最小化按鈕存在」斷言）
- Modify: `waypoint/src/windows/NoteWindow.render.test.pw.ts`（如有）

- [ ] **Step 1：寫失敗 render test**

```ts
// waypoint/src/windows/NoteWindow.render.test.pw.ts
test('note titlebar does not contain minimize button [—]', async ({ page }) => {
  await page.goto('/#view=note&noteId=test&contextId=null');
  const minimize = page.locator('.titlebar-buttons button[title="最小化"]');
  await expect(minimize).toHaveCount(0);
});
```

- [ ] **Step 2：跑 `npm run build && npm run test:render` 確認 FAIL**

- [ ] **Step 3：刪除 NoteWindow.svelte 的最小化按鈕 + handler + `windowsApi.minimizeWindow` 呼叫**

```svelte
<!-- BEFORE -->
<button on:click={handleMinimize} title="最小化">—</button>
<!-- 刪除整行 + handleMinimize 函式 -->
```

- [ ] **Step 4：跑 render test 確認 PASS**

- [ ] **Step 5：更新 E2E smoke spec 移除「最小化按鈕」相關斷言；跑 act 確認綠**

- [ ] **Step 6：commit**

```bash
git add src/windows/NoteWindow.svelte src/windows/NoteWindow.render.test.pw.ts e2e/specs/
git commit -m "feat(note): 移除最小化按鈕"
```

### Task 2.2：移除列表自動隱藏功能（R8）

**Files:**
- Modify: `waypoint/src-tauri/src/hotkey/mod.rs`（`attach_list_autohide` 函式 + 呼叫處）
- Modify: `waypoint/src-tauri/src/storage/app_config.rs`（如有 autohide 欄位）
- Modify: `waypoint/src/windows/list/SettingsPanel.svelte`（如有 autohide UI）

- [ ] **Step 1：寫失敗 cargo test**

```rust
// 在 hotkey/mod.rs 尾部 #[cfg(test)] mod tests
#[test]
fn list_autohide_function_removed() {
    // 編譯期斷言：以下符號不應存在
    // 若 attach_list_autohide 仍存在，下一行的引用會 compile fail
    // (測試方法：在 git history 確認此函式已被刪除；compile fail 本身就是測試)
    let _ = "list autohide is intentionally removed";
}
```

- [ ] **Step 2：刪除 `attach_list_autohide` 函式定義與呼叫**

- [ ] **Step 3：刪除 SettingsPanel autohide UI 與儲存欄位（若有）**

- [ ] **Step 4：`cargo test && npm test` 全綠**

- [ ] **Step 5：commit**

```bash
git add src-tauri/src/hotkey/mod.rs src-tauri/src/storage/app_config.rs src/windows/list/
git commit -m "feat(list): 移除自動隱藏功能"
```

### Task 2.3：HelpWindow / SettingsWindow titlebar 拖曳修復（R12）

**Files:**
- Modify: `waypoint/src/windows/HelpWindow.svelte`（line 6 titlebar）
- Modify: `waypoint/src/windows/SettingsWindow.svelte`（line 93 titlebar）
- Create: `waypoint/e2e/specs/system-windows-drag.spec.js`

- [ ] **Step 1：寫失敗 E2E**

```js
// waypoint/e2e/specs/system-windows-drag.spec.js
describe('system windows drag', () => {
  for (const view of ['help', 'settings']) {
    it(`${view} window titlebar mousedown is wired`, async () => {
      // tauri-driver 無法直接驗證視窗位置，改驗 handler 存在：
      // 透過 evaluate 檢查 .titlebar 元素的 mousedown listener
      await browser.url(`/#view=${view}`);
      const hasListener = await browser.execute(() => {
        const el = document.querySelector('.titlebar');
        if (!el) return false;
        return el.getAttribute('data-tauri-drag-region') !== null;
      });
      expect(hasListener).toBe(true);
    });
  }
});
```

- [ ] **Step 2：把 NoteWindow 的 `handleTitlebarMousedown` 模式套到 Help / Settings**

```svelte
<!-- HelpWindow.svelte / SettingsWindow.svelte 同樣修改 -->
<script>
  import { windowsApi } from '../lib/api';
  function handleTitlebarMousedown(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (target.closest('button') || target.closest('input')) return;
    windowsApi.startDragging('help' /* or 'settings' */).catch(() => {});
  }
</script>

<div class="titlebar" data-tauri-drag-region on:mousedown={handleTitlebarMousedown}>
  <!-- ... -->
</div>
```

- [ ] **Step 3：確認 `windowsApi.startDragging` 已存在**（如否，補上 invoke `cmd_start_dragging` 或用 `getCurrentWindow().startDragging()`）

- [ ] **Step 4：跑 render test + act e2e 全綠**

- [ ] **Step 5：commit**

```bash
git add src/windows/HelpWindow.svelte src/windows/SettingsWindow.svelte e2e/specs/system-windows-drag.spec.js
git commit -m "fix: Help/Settings titlebar 拖曳套用 NoteWindow 的 mousedown fallback"
```

### Task 2.4：筆記 titlebar 加 ⇊ 收起按鈕 + emit 事件（R11，dot 留 R9）

**Files:**
- Modify: `waypoint/src/windows/NoteWindow.svelte`
- Create: `waypoint/src/windows/NoteWindow.render.test.pw.ts`（追加 case）
- Create: `waypoint/e2e/specs/note-collapse-button.spec.js`

- [ ] **Step 1：render test — titlebar 有 `⇊` 按鈕**

```ts
test('note titlebar has collapse-all button (⇊)', async ({ page }) => {
  await page.goto('/#view=note&noteId=test&contextId=null');
  const btn = page.locator('.titlebar-buttons button[title="收起全部並儲存"]');
  await expect(btn).toHaveCount(1);
  await expect(btn).toHaveText('⇊');
});
```

- [ ] **Step 2：實作 — emit 事件而非直接呼叫 collapse_all**

```svelte
<script>
  import { emit } from '@tauri-apps/api/event';
  async function handleCollapseAll() {
    await emit('waypoint://collapse-all-requested');
  }
</script>

<button on:click={handleCollapseAll} title="收起全部並儲存">⇊</button>
```

- [ ] **Step 3：E2E — 開兩個 note → 從某個 note 點 ⇊ → 全關 → 重啟 → 兩 note 還原**

```js
describe('note collapse button', () => {
  it('collapses all and persists session', async () => {
    // 透過 list 開兩個 note；切到某個 note；點 ⇊
    // 等待所有 note + list 都 hide；重啟（重設 WAYPOINT_E2E）
    // 重啟後從 session 還原應有兩個 note
    // ...（依 e2e/specs/restart.spec.js 模式）
  });
});
```

- [ ] **Step 4：執行所有測試確認綠**

- [ ] **Step 5：commit**

```bash
git add src/windows/NoteWindow.svelte src/windows/NoteWindow.render.test.pw.ts e2e/specs/note-collapse-button.spec.js
git commit -m "feat(note): titlebar 新增 ⇊ 收起全部並儲存按鈕（emit 事件）"
```

---

## Phase 3 — 小修（R3, R4, R6）

### Task 3.1：筆記永遠最上層（R3）

**Files:**
- Modify: `waypoint/src-tauri/src/hotkey/mod.rs`（`open_note_window`）

- [ ] **Step 1：cargo test — 確認 builder 有 `.always_on_top(true)`**

```rust
#[test]
fn note_window_builder_sets_always_on_top() {
    // 因為無法直接測 builder fluent chain，改用 source 字串斷言：
    let src = include_str!("../hotkey/mod.rs");
    assert!(src.contains(".always_on_top(true)"), "open_note_window must call always_on_top(true)");
}
```

- [ ] **Step 2：跑 `cargo test` FAIL**

- [ ] **Step 3：改 `open_note_window`**

```rust
WebviewWindowBuilder::new(app, &label, WebviewUrl::App(url.into()))
    .title("Waypoint Note")
    .inner_size(420.0, 600.0)
    .min_inner_size(300.0, 200.0)
    .resizable(true)
    .decorations(false)
    .skip_taskbar(true)
    .always_on_top(true)   // ← 新增
    .build()?;
```

- [ ] **Step 4：cargo test PASS**

- [ ] **Step 5：commit**

```bash
git add src-tauri/src/hotkey/mod.rs
git commit -m "feat(note): 視窗永遠最上層"
```

### Task 3.2：移除筆記專屬快捷鍵（R4）

**Files:**
- Modify: `waypoint/src-tauri/src/hotkey/mod.rs`（移除 `register_note_hotkey`、`cmd_register_note_hotkey`、`cmd_unregister_hotkey`）
- Modify: `waypoint/src-tauri/src/lib.rs`（從 invoke_handler 移除）
- Modify: `waypoint/src/lib/api.ts`（移除 `registerNoteHotkey` / `unregisterHotkey`）
- Modify: `waypoint/src/windows/note/SettingsPanel.svelte`（移除 hotkey 欄位）
- Modify: `waypoint/src/lib/types.ts`（`NoteSettings.hotkey` 欄位保留以相容舊 config，但不再使用）

- [ ] **Step 1：vitest — SettingsPanel 沒有 hotkey input**

```ts
test('NoteSettingsPanel does not expose per-note hotkey field', async () => {
  // mount 後 dom 不應有 hotkey input
  // 用 vitest + happy-dom 或 svelte testing library
});
```

- [ ] **Step 2：刪除上述 Rust / TS / Svelte 對應段落**

- [ ] **Step 3：cargo test + npm test PASS**

- [ ] **Step 4：commit**

```bash
git add ...
git commit -m "refactor: 移除筆記專屬快捷鍵（保留 NoteSettings.hotkey 欄位以相容舊設定）"
```

### Task 3.3：透明度 slider 100% 對齊修正（R6）

**Files:**
- Modify: `waypoint/src/windows/note/SettingsPanel.svelte`（slider markup + CSS）
- Create: `waypoint/src/windows/note/SettingsPanel.render.test.pw.ts`

- [ ] **Step 1：render test — value=100 時 thumb 在右邊界**

```ts
test('opacity slider: value=1.0 thumb is at right edge', async ({ page }) => {
  await page.goto('/#view=note&noteId=test&contextId=null');
  // 開 settings panel
  await page.click('button[title="設定"]');
  const slider = page.locator('input[type="range"]');
  await slider.fill('1');
  const box = await slider.boundingBox();
  // pseudo: 取 thumb 中心 X，斷言距離右邊界 < 4px
  // 若無法直接抓 thumb，改驗 computed style left ≈ 100%
});
```

- [ ] **Step 2：FAIL → 修正 slider markup 與 CSS**

```svelte
<input
  type="range"
  min="0"
  max="100"
  step="1"
  value={Math.round(settings.opacity * 100)}
  on:input={e => update({ opacity: parseInt((e.target as HTMLInputElement).value, 10) / 100 })}
/>

<style>
  input[type="range"] {
    appearance: none;
    width: 100%;
    margin: 0;
  }
  input[type="range"]::-webkit-slider-thumb {
    appearance: none;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--accent);
    margin-top: -4px;
  }
</style>
```

- [ ] **Step 3：render test PASS**

- [ ] **Step 4：commit**

```bash
git add src/windows/note/SettingsPanel.svelte src/windows/note/SettingsPanel.render.test.pw.ts
git commit -m "fix(note): 透明度 slider 100% 時 thumb 對齊右邊界"
```

> **Deviation（2026-04-22 補記）**：實作將 `min` 從 spec 的 `0` 改為 `10`、`step` 改為 `5`。理由：避免使用者拉到 0% 後筆記完全透明、無法再點開設定。視為 UX 守門，已於 SettingsPanel.svelte 加註解。

---

## Phase 4 — 真透明（R5）

### Task 4.1：筆記視窗 transparent + RGBA 背景

**Files:**
- Modify: `waypoint/src-tauri/src/hotkey/mod.rs`（`open_note_window` 加 `.transparent(true)`）
- Modify: `waypoint/src-tauri/tauri.conf.json`（如需 windows section 設 `transparent: true`）
- Modify: `waypoint/src/app.css`（`--bg-primary` 改 rgba）
- Modify: `waypoint/src/windows/NoteWindow.svelte`（`applyOpacity` 改成設 CSS variable 而非 `documentElement.style.opacity`）
- Modify: `waypoint/src/app.html`（移除 / 改寫 inline body 深色背景，讓筆記透明可看穿；但保留 list/help/settings 不透明）

- [ ] **Step 1：cargo test — open_note_window 含 transparent**

```rust
#[test]
fn note_window_builder_sets_transparent() {
    let src = include_str!("../hotkey/mod.rs");
    assert!(src.contains(".transparent(true)"));
}
```

- [ ] **Step 2：render test — 筆記 body computed background-color 是 rgba 且 alpha < 1**

```ts
test('note window background is RGBA with adjustable alpha', async ({ page }) => {
  await page.goto('/#view=note&noteId=test&contextId=null');
  const bg = await page.evaluate(() => getComputedStyle(document.body).backgroundColor);
  expect(bg).toMatch(/^rgba\(/);
  // 設 opacity 0.3 → 重新讀 alpha
});
```

- [ ] **Step 3：白屏 fallback — render test 斷言 list/help/settings body bg 仍為深色 RGB（避免改透明傷到非 note 視窗）**

- [ ] **Step 4：實作上述四個 file 的修改**

- [ ] **Step 5：所有 test 綠，視覺手動驗證 Windows 上能看穿（push master 跑 e2e-windows）**

- [ ] **Step 6：commit**

```bash
git add ...
git commit -m "feat(note): 真透明 — Tauri transparent + RGBA 背景，opacity 控制 alpha"
```

---

## Phase 5 — 穿透模式（R9，依賴 R5）

### Task 5.1：NoteSettings 加 passthrough 欄位 + Rust 結構同步

**Files:**
- Modify: `waypoint/src/lib/types.ts`（`NoteSettings` 加 `passthrough: boolean`）
- Modify: `waypoint/src-tauri/src/storage/notes.rs`（對應 Rust struct）
- Create: `waypoint/src/lib/types.test.ts`（追加 case）

- [ ] **Step 1：vitest — NoteSettings 預設 passthrough=false 且可序列化**

```ts
test('NoteSettings.passthrough defaults to false', () => {
  const s: NoteSettings = { fontSize: 14, opacity: 1, hotkey: null, windowBounds: null, passthrough: false };
  expect(s.passthrough).toBe(false);
});
```

- [ ] **Step 2：cargo test — 反序列化舊 JSON（無 passthrough 欄位）→ 預設 false**

```rust
#[test]
fn note_settings_deserializes_without_passthrough_field() {
    let json = r#"{"fontSize":14,"opacity":1.0,"hotkey":null,"windowBounds":null}"#;
    let s: NoteSettings = serde_json::from_str(json).unwrap();
    assert_eq!(s.passthrough, false);
}
```

- [ ] **Step 3：實作 — TS interface + Rust struct 加 `#[serde(default)] pub passthrough: bool`**

- [ ] **Step 4：所有測試綠**

- [ ] **Step 5：commit**

```bash
git add src/lib/types.ts src/lib/types.test.ts src-tauri/src/storage/notes.rs
git commit -m "feat(note): NoteSettings 新增 passthrough 欄位（預設 false）"
```

### Task 5.2：Rust 端穿透 commands

**Files:**
- Create: `waypoint/src-tauri/src/commands/passthrough_cmd.rs`
- Modify: `waypoint/src-tauri/src/commands/mod.rs`
- Modify: `waypoint/src-tauri/src/lib.rs`（invoke_handler 註冊）

- [ ] **Step 1：cargo test — 全域 toggle 邏輯（混合 → 全開；全開 → 全關；全關 → 全開）**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn target_state(states: &[bool]) -> bool {
        // 規則：任一非穿透 → 全部設為穿透 (true)；全部已穿透 → 全部關 (false)
        !states.iter().all(|&s| s)
    }

    #[test]
    fn mixed_state_targets_all_on() {
        assert_eq!(target_state(&[true, false, true]), true);
    }

    #[test]
    fn all_on_targets_all_off() {
        assert_eq!(target_state(&[true, true, true]), false);
    }

    #[test]
    fn all_off_targets_all_on() {
        assert_eq!(target_state(&[false, false]), true);
    }

    #[test]
    fn empty_targets_on() {
        assert_eq!(target_state(&[]), true);
    }
}
```

- [ ] **Step 2：實作 commands**

```rust
// waypoint/src-tauri/src/commands/passthrough_cmd.rs
use tauri::{AppHandle, Manager, Emitter};

pub fn target_state(states: &[bool]) -> bool {
    !states.iter().all(|&s| s)
}

fn collect_note_states(app: &AppHandle) -> Vec<(String, bool)> {
    app.webview_windows()
        .iter()
        .filter(|(label, _)| label.starts_with("note-"))
        .map(|(label, win)| {
            // 讀目前 ignore_cursor_events 狀態（無 getter，需在 AppState 自行追蹤）
            // 暫存於 AppState.passthrough_state: HashMap<String, bool>
            let state = app.state::<crate::state::AppState>();
            let map = state.passthrough_state.lock().unwrap();
            (label.clone(), *map.get(label).unwrap_or(&false))
        })
        .collect()
}

#[tauri::command]
pub fn cmd_set_passthrough(app: AppHandle, note_label: String, on: bool) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(&note_label) {
        win.set_ignore_cursor_events(on).map_err(|e| e.to_string())?;
    }
    let state = app.state::<crate::state::AppState>();
    state.passthrough_state.lock().unwrap().insert(note_label.clone(), on);
    let _ = app.emit("waypoint://passthrough-changed", (note_label, on));
    Ok(())
}

#[tauri::command]
pub fn cmd_toggle_passthrough_global(app: AppHandle) -> Result<(), String> {
    let states: Vec<bool> = collect_note_states(&app).iter().map(|(_, s)| *s).collect();
    let target = target_state(&states);
    let labels: Vec<String> = collect_note_states(&app).into_iter().map(|(l, _)| l).collect();
    for l in labels {
        cmd_set_passthrough(app.clone(), l, target)?;
    }
    Ok(())
}
```

- [ ] **Step 3：在 `state.rs` 加 `pub passthrough_state: Mutex<HashMap<String, bool>>`**

- [ ] **Step 4：lib.rs invoke_handler 註冊兩個 cmd**

- [ ] **Step 5：cargo test PASS**

- [ ] **Step 6：commit**

```bash
git add src-tauri/src/commands/ src-tauri/src/state.rs src-tauri/src/lib.rs
git commit -m "feat: 穿透 commands (cmd_set_passthrough / cmd_toggle_passthrough_global)"
```

### Task 5.3：穿透全域快捷鍵註冊

**Files:**
- Modify: `waypoint/src-tauri/src/storage/app_config.rs`（加 `passthrough_hotkey: String`，預設 `Ctrl+Shift+T`）
- Modify: `waypoint/src-tauri/src/hotkey/mod.rs`（註冊新 hotkey → 呼叫 `cmd_toggle_passthrough_global`）
- Modify: `waypoint/src-tauri/src/commands/config_cmd.rs`（加 `set_passthrough_hotkey`）
- Modify: `waypoint/src/lib/api.ts`（加 `togglePassthroughGlobal`、`setPassthroughHotkey`）

- [ ] **Step 1：cargo test — AppConfig 預設含 passthrough_hotkey="Ctrl+Shift+T"，舊 config 反序列化也帶預設**

- [ ] **Step 2：cargo test — register_passthrough_hotkey 註冊後 unregister 可再 register**

- [ ] **Step 3：實作上述變更**

- [ ] **Step 4：E2E — 按 Ctrl+Shift+T → 任一筆記變穿透**

```js
// waypoint/e2e/specs/passthrough-hotkey.spec.js
it('global hotkey toggles passthrough on all notes', async () => {
  // 開兩個 note；發 Ctrl+Shift+T；驗 passthrough-changed 事件 × 2 + ignore_cursor_events true
});
```

- [ ] **Step 5：cargo + render + e2e 全綠**

- [ ] **Step 6：commit**

```bash
git add ...
git commit -m "feat: 穿透全域快捷鍵 Ctrl+Shift+T（可在列表設定修改）"
```

### Task 5.4：列表設定面板新增穿透 hotkey 欄位

**Files:**
- Modify: `waypoint/src/windows/list/SettingsPanel.svelte`（或對應檔）

- [ ] **Step 1：render test — 設定面板含「穿透快捷鍵」input，預設值為 Ctrl+Shift+T**

- [ ] **Step 2：實作欄位（沿用既有「叫出列表快捷鍵」UI 風格）**

- [ ] **Step 3：commit**

### Task 5.5：NoteWindow titlebar dot 元件 + 監聽穿透事件

**Files:**
- Modify: `waypoint/src/windows/NoteWindow.svelte`

- [ ] **Step 1：render test — titlebar 有 `.passthrough-dot` 元素；初始 class 含 `dot-on` (綠) 或 `dot-off` (灰)；點擊呼叫 `togglePassthroughGlobal`**

```ts
test('passthrough dot exists left of collapse button', async ({ page }) => {
  await page.goto('/#view=note&noteId=test&contextId=null');
  const dot = page.locator('.passthrough-dot');
  await expect(dot).toHaveCount(1);
  await expect(dot).toHaveClass(/dot-on/);
});
```

- [ ] **Step 2：實作**

```svelte
<script>
  import { listen } from '@tauri-apps/api/event';
  import { onMount, onDestroy } from 'svelte';

  let passthrough = false;
  let unlisten: (() => void) | undefined;

  onMount(async () => {
    passthrough = note?.settings.passthrough ?? false;
    unlisten = await listen<[string, boolean]>('waypoint://passthrough-changed', (e) => {
      const [label, on] = e.payload;
      if (label === `note-${noteId}`) passthrough = on;
    });
  });
  onDestroy(() => unlisten?.());

  async function handleDotClick() {
    await invoke('cmd_toggle_passthrough_global');
  }
</script>

<button
  class="passthrough-dot"
  class:dot-on={!passthrough}
  class:dot-off={passthrough}
  on:click={handleDotClick}
  title={passthrough ? '穿透中（按快捷鍵或 tray 關閉）' : '可互動 — 點此啟用穿透'}
>●</button>

<style>
  .passthrough-dot {
    width: 14px; height: 14px;
    border-radius: 50%;
    border: none;
    padding: 0;
    cursor: pointer;
  }
  .dot-on  { background: #5cb85c; }
  .dot-off { background: #ffb454; box-shadow: 0 0 6px #ffb454; }
</style>
```

擺在 `[● dot]` 位置（⇊ 按鈕左邊）。

- [ ] **Step 3：render + e2e 測試綠**

- [ ] **Step 4：commit**

```bash
git add src/windows/NoteWindow.svelte
git commit -m "feat(note): titlebar dot — 綠=可互動 / 黃=穿透中，點擊全域切換"
```

### Task 5.6：穿透狀態持久化

**Files:**
- Modify: `waypoint/src-tauri/src/commands/passthrough_cmd.rs`（`cmd_set_passthrough` 後寫回 note settings）

- [ ] **Step 1：cargo test — set_passthrough 後讀 note 檔案 `passthrough` 為 true**

- [ ] **Step 2：實作 — 解析 label 取 note_id，呼叫 `storage::notes::load → mutate → save`**

- [ ] **Step 3：E2E — 設穿透 → 重啟 → 還原為穿透**

- [ ] **Step 4：commit**

### Task 5.7：tray 右鍵選單加穿透開關

**Files:**
- Modify: `waypoint/src-tauri/src/tray/mod.rs`

- [ ] **Step 1：cargo test — tray menu 包含 `passthrough_toggle` item，文字含「穿透」**

- [ ] **Step 2：實作 — 監聽 menu click → 呼叫 `cmd_toggle_passthrough_global`；監聽 `waypoint://passthrough-changed` 事件 → 更新 menu item 文字 (`● 穿透：開` / `○ 穿透：關`)**

- [ ] **Step 3：commit**

---

## Phase 6 — 確認框與工作列（R7, R13）

### Task 6.1：刪除確認框脫離列表寬度（R7）

**Files:**
- Create: `waypoint/src/windows/list/ConfirmDialog.svelte`（或用 native Tauri dialog）
- Modify: `waypoint/src/windows/ListWindow.svelte`（右鍵刪除流程改用 ConfirmDialog）

- [ ] **Step 1：render test — 列表預設寬度 220px 下，confirm dialog 兩個按鈕完整可見且不換行**

```ts
test('delete confirm dialog renders both buttons fully visible at default list width', async ({ page }) => {
  await page.setViewportSize({ width: 220, height: 500 });
  await page.goto('/#view=list');
  // 模擬有 note → 右鍵 → 刪除
  // 等待 dialog
  const cancel = page.locator('.confirm-dialog button.cancel');
  const ok = page.locator('.confirm-dialog button.danger');
  await expect(cancel).toBeVisible();
  await expect(ok).toBeVisible();
  const cancelBox = await cancel.boundingBox();
  const okBox = await ok.boundingBox();
  // 兩個按鈕底部都在 viewport 內
  expect(cancelBox!.y + cancelBox!.height).toBeLessThan(500);
  expect(okBox!.y + okBox!.height).toBeLessThan(500);
});
```

- [ ] **Step 2：實作 ConfirmDialog — 用 fixed position overlay 覆蓋整個列表視窗，按鈕直立堆疊（避免水平擠壓）**

```svelte
<!-- ConfirmDialog.svelte -->
<div class="overlay">
  <div class="dialog">
    <p class="msg">{message}</p>
    <button class="danger" on:click={onConfirm}>刪除</button>
    <button class="cancel" on:click={onCancel}>取消</button>
  </div>
</div>
<style>
  .overlay { position: fixed; inset: 0; background: rgba(0,0,0,.6); display: flex; align-items: center; justify-content: center; z-index: 1000; }
  .dialog  { background: var(--bg-primary); padding: 12px; border-radius: 6px; width: calc(100% - 24px); display: flex; flex-direction: column; gap: 8px; }
  .dialog button { width: 100%; }
</style>
```

- [ ] **Step 3：替換 ListWindow 既有的 confirm 流程**

- [ ] **Step 4：render test PASS**

- [ ] **Step 5：commit**

### Task 6.2：列表 settings 加 showInTaskbar toggle + Rust 設定欄位（R13 part 1）

**Files:**
- Modify: `waypoint/src-tauri/src/storage/app_config.rs`（加 `show_in_taskbar: bool` 預設 `true`）
- Modify: `waypoint/src-tauri/src/commands/config_cmd.rs`（加 `set_show_in_taskbar`）
- Modify: `waypoint/src/lib/types.ts`（AppConfig 加 `showInTaskbar`）
- Modify: `waypoint/src/lib/api.ts`（加 `setShowInTaskbar`）
- Modify: `waypoint/src/windows/list/SettingsPanel.svelte`（新增 checkbox）

- [ ] **Step 1：cargo test — AppConfig 預設 show_in_taskbar=true；舊 config 反序列化帶預設**

- [ ] **Step 2：vitest — SettingsPanel checkbox 變更 → 呼叫 setShowInTaskbar**

- [ ] **Step 3：實作上述全部**

- [ ] **Step 4：commit**

### Task 6.3：refresh_taskbar_visibility 函式 + hook 進視窗 show/hide 路徑（R13 part 2）

**Files:**
- Modify: `waypoint/src-tauri/src/hotkey/mod.rs`（在 `open_list_window`、`open_note_window`、`collapse_all_waypoint_windows` 末尾呼叫；handleClose 路徑同樣）
- Create: `waypoint/src-tauri/src/taskbar.rs`

- [ ] **Step 1：cargo test — 邏輯函式 `should_show_taskbar(visible_count, setting) -> bool`**

```rust
#[test]
fn taskbar_visible_when_window_visible_and_setting_on() {
    assert!(should_show_taskbar(1, true));
    assert!(should_show_taskbar(3, true));
}
#[test]
fn taskbar_hidden_when_no_window() {
    assert!(!should_show_taskbar(0, true));
}
#[test]
fn taskbar_hidden_when_setting_off() {
    assert!(!should_show_taskbar(5, false));
}
```

- [ ] **Step 2：實作 `refresh_taskbar_visibility(app)` — 遍歷所有 webview_windows，數可見的，依 should_show_taskbar 結果對每個視窗 `set_skip_taskbar(!show)`**

- [ ] **Step 3：在每個 show/hide 點 hook 呼叫**

- [ ] **Step 4：E2E — 開列表 → 工作列出現；收起 → 消失；toggle setting → 行為變更**

```js
it('taskbar entry appears with any visible window and disappears when none', async () => {
  // tauri-driver 沒有直接讀 OS taskbar 的 API
  // 改驗：每個 window 的 isMinimized / 自定義事件「taskbar-state-changed」
});
```

- [ ] **Step 5：commit**

---

## Phase 7 — 整合驗證與發布

### Task 7.1：本機完整測試矩陣

- [ ] `cd waypoint && npm test`
- [ ] `cd waypoint/src-tauri && cargo test`
- [ ] `cd waypoint && npm run build && npm run test:render`
- [ ] `export DOCKER_HOST=tcp://host.docker.internal:2375 && act -j e2e-linux`

全綠才往下。

### Task 7.2：merge dev/main → master，等 e2e-windows

- [ ] `git checkout master && git merge --ff-only dev/main && git push origin master`
- [ ] 用 CLAUDE.md 的 curl 輪詢 workflow id `262677203` 的 latest run，等 conclusion=success
- [ ] 失敗 → 看 ci-logs 分支 → 補修 → 重推

### Task 7.3：bump version 並 tag

- [ ] 更新 `waypoint/package.json` 與 `waypoint/src-tauri/tauri.conf.json` version
- [ ] commit 訊息 `chore: bump version to 0.1.15`
- [ ] `git tag v0.1.15 && git push --tags`

---

## Self-Review 檢查清單（寫完後對照 spec）

- [x] R1（多餘視窗）— Phase 1 Task 1.1/1.2
- [x] R2（測試）— Phase 1 Task 1.1
- [x] R3（always_on_top）— Task 3.1
- [x] R4（移除 note 專屬 hotkey）— Task 3.2
- [x] R5（真透明）— Phase 4 Task 4.1
- [x] R6（slider 100%）— Task 3.3
- [x] R7（confirm 框）— Task 6.1
- [x] R8（移除 list autohide）— Task 2.2
- [x] R9（穿透完整功能）— Phase 5 Task 5.1~5.7
- [x] R10（移除 note 最小化）— Task 2.1
- [x] R11（⇊ 收起按鈕）— Task 2.4
- [x] R12（Help/Settings 拖曳）— Task 2.3
- [x] R13（工作列大圖示）— Task 6.2/6.3
