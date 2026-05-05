# Plan C — 穿透模式隱藏 chrome Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 穿透模式開啟時：筆記視窗 titlebar 完全隱藏（保留 title input + content）；列表視窗自我 hide。關閉穿透時：titlebar 還原、list 視「進入穿透前的狀態」恢復。

**Architecture:** Rust 端在 `cmd_toggle_passthrough_global` 切換時 snapshot list 開啟狀態；emit 新事件 `waypoint://passthrough-globally-on` / `waypoint://passthrough-globally-off`。前端：NoteWindow 條件渲染 titlebar；ListWindow 收到 on 事件 hide、off 事件視 snapshot 重 show。

**Tech Stack:** Tauri command + emit、Svelte 條件渲染、AtomicBool state。

**Dependencies:** 建議在 Plan A 之後（NoteWindow titlebar 結構已穩定），與 Plan B 互不衝突。

---

## File Structure

| File | Status | Responsibility |
|---|---|---|
| `src-tauri/src/state.rs` | Modify | 加 `pre_passthrough_list_open: AtomicBool` |
| `src-tauri/src/commands/passthrough_cmd.rs` | Modify | toggle global 前 snapshot；emit globally-on/off 事件 |
| `src-tauri/src/lib.rs` | No change（事件由 passthrough_cmd 自行發） |  |
| `src/windows/NoteWindow.svelte` | Modify | passthrough=true 時不渲染 `<DraggableTitlebar>` |
| `src/windows/ListWindow.svelte` | Modify | 加 listen `passthrough-globally-on`/`off`，自我 hide/show |
| `src/windows/NoteWindow.passthrough.render.test.pw.ts` | Modify | 加 case：passthrough true 時 .titlebar 不存在、.title-input 仍在 |
| `e2e/specs/passthrough-chrome.spec.js` | Create | toggle on → titlebar 不見、list hide；toggle off → 還原 |

---

## Task 1: state 加 pre_passthrough_list_open

**Files:**
- Modify: `src-tauri/src/state.rs`

- [ ] **Step 1: 加欄位**

state.rs `AppState` struct 加：

```rust
pub pre_passthrough_list_open: AtomicBool,
```

`Default::default` 加：

```rust
pre_passthrough_list_open: AtomicBool::new(false),
```

- [ ] **Step 2: cargo check**

```bash
cd src-tauri && cargo check 2>&1 | tail -5
```

Expected: ok

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/state.rs
git commit -m "refactor(passthrough): add pre_passthrough_list_open snapshot field"
```

---

## Task 2: passthrough_cmd 切換時 snapshot + emit globally events

**Files:**
- Modify: `src-tauri/src/commands/passthrough_cmd.rs`

- [ ] **Step 1: Write failing test**

passthrough_cmd.rs 既有的 `target_state` 測試保留。新增：

```rust
#[cfg(test)]
mod global_tests {
    // pre_passthrough_list_open 行為的 unit test 不需 Tauri runtime；
    // 用簡易 helper 直接測試 snapshot 邏輯：當 target=on 時 snapshot 取 list_open；
    // 當 target=off 時讀回 snapshot。
    use std::sync::atomic::{AtomicBool, Ordering};

    fn snapshot_then_apply(list_open: bool, going_on: bool, snapshot: &AtomicBool) -> bool {
        if going_on {
            snapshot.store(list_open, Ordering::SeqCst);
            // 進入穿透：list 應該 hide → 回傳 should_show_list=false
            false
        } else {
            // 退出穿透：依 snapshot 還原
            snapshot.load(Ordering::SeqCst)
        }
    }

    #[test]
    fn going_on_snapshots_list_state_and_returns_hide() {
        let snap = AtomicBool::new(false);
        let show = snapshot_then_apply(true, true, &snap);
        assert!(!show);
        assert!(snap.load(Ordering::SeqCst));
    }

    #[test]
    fn going_off_restores_from_snapshot() {
        let snap = AtomicBool::new(true);
        let show = snapshot_then_apply(false, false, &snap);
        assert!(show);
    }

    #[test]
    fn going_off_when_was_closed_stays_closed() {
        let snap = AtomicBool::new(false);
        let show = snapshot_then_apply(false, false, &snap);
        assert!(!show);
    }
}
```

- [ ] **Step 2: 改 cmd_toggle_passthrough_global**

```rust
#[tauri::command]
pub fn cmd_toggle_passthrough_global(app: AppHandle) -> Result<(), String> {
    let labels = collect_note_labels(&app);
    let states: Vec<bool> = {
        let state = app.state::<crate::state::AppState>();
        let map = state.passthrough_state.lock().unwrap();
        labels.iter().map(|l| *map.get(l).unwrap_or(&false)).collect()
    };
    let target = target_state(&states);
    let state = app.state::<crate::state::AppState>();
    if target {
        // going on：snapshot 當下 list_open，並廣播 globally-on
        let list_open = *state.list_window_open.lock().unwrap();
        state.pre_passthrough_list_open.store(list_open, std::sync::atomic::Ordering::SeqCst);
        let _ = app.emit("waypoint://passthrough-globally-on", ());
    } else {
        // going off：依 snapshot 還原
        let should_show = state.pre_passthrough_list_open.load(std::sync::atomic::Ordering::SeqCst);
        let _ = app.emit("waypoint://passthrough-globally-off", should_show);
    }
    for l in labels {
        cmd_set_passthrough(app.clone(), l, target)?;
    }
    Ok(())
}
```

- [ ] **Step 3: Run cargo test**

```bash
cd src-tauri && cargo test --lib commands::passthrough_cmd
```

Expected: PASS（target_state 既有 4 個 + global_tests 新加 3 個）

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/passthrough_cmd.rs
git commit -m "feat(passthrough): snapshot list state on global toggle, emit globally on/off"
```

---

## Task 3: NoteWindow 條件隱藏 titlebar

**Files:**
- Modify: `src/windows/NoteWindow.svelte`
- Modify: `src/windows/NoteWindow.passthrough.render.test.pw.ts`

- [ ] **Step 1: Write failing test**

把 `src/windows/NoteWindow.passthrough.render.test.pw.ts` 末尾加（保留既有 case）：

```ts
test('passthrough=true 時 .titlebar 不存在，title-input 仍在', async ({ page }) => {
  await page.addInitScript(() => {
    (window as any).__TAURI_INTERNALS__ = {
      metadata: { currentWindow: { label: 'note-x' }, currentWebview: { label: 'note-x', windowLabel: 'note-x' } },
      invoke: (cmd: string) => {
        if (cmd === 'read_note') return Promise.resolve({
          id: 'x', contextId: null, title: 'T', content: 'hi',
          settings: { fontSize: 14, opacity: 1, hotkey: null, windowBounds: null, passthrough: true },
        });
        if (cmd === 'get_transparent_includes_text') return Promise.resolve(true);
        if (cmd === 'plugin:event|listen') return Promise.resolve(0);
        return Promise.resolve(null);
      },
      transformCallback: () => 0, unregisterCallback: () => {}, convertFileSrc: (s: string) => s,
    };
  });
  await page.goto('http://localhost:4173/#view=note&noteId=x');
  await page.waitForLoadState('networkidle');
  await expect(page.locator('.draggable-titlebar')).toHaveCount(0);
  await expect(page.locator('.title-input')).toBeVisible();
  await expect(page.locator('.editor-wrap')).toBeVisible();
});
```

- [ ] **Step 2: Run test, verify FAIL**

```bash
npm run build && npx playwright test src/windows/NoteWindow.passthrough.render.test.pw.ts
```

Expected: 新 case FAIL（titlebar 還是存在）

- [ ] **Step 3: 改 NoteWindow.svelte 模板**

把目前的 `<DraggableTitlebar label={...}>...</DraggableTitlebar>` 包進 `{#if !passthrough}`：

```svelte
{#if !passthrough}
  <DraggableTitlebar label={`note-${noteId}`}>
    <span class="note-title" data-tauri-drag-region>
      {title || "Untitled"}
      {#if contextId === null}<GlobeIcon size={12} />{/if}
    </span>
    <TitlebarOpacitySlider ... />
    <div class="titlebar-buttons">...</div>
  </DraggableTitlebar>
{/if}
```

title `<input>` 與 editor-area 不變（保留）。

- [ ] **Step 4: Run test, verify PASS**

```bash
npx playwright test src/windows/NoteWindow.passthrough.render.test.pw.ts
```

Expected: 全綠

- [ ] **Step 5: Commit**

```bash
git add src/windows/NoteWindow.svelte src/windows/NoteWindow.passthrough.render.test.pw.ts
git commit -m "feat(passthrough): hide note titlebar when passthrough on"
```

---

## Task 4: list hide on / restore via Rust + 前端 listener

**架構決策**：list 還原邏輯放 Rust 端最乾淨。`cmd_toggle_passthrough_global` going off 時直接呼叫 `crate::hotkey::open_list_window(&app)` 視 snapshot 而定。前端 list 只需收 globally-on 自我 hide。

**Files:**
- Modify: `src-tauri/src/commands/passthrough_cmd.rs`
- Modify: `src/windows/ListWindow.svelte`

- [ ] **Step 1: 改 cmd_toggle_passthrough_global going off 直接 open list**

把 Task 2 寫進去的 `else` 段改成：

```rust
} else {
    let should_show = state.pre_passthrough_list_open.load(std::sync::atomic::Ordering::SeqCst);
    if should_show {
        let _ = crate::hotkey::open_list_window(&app);
    }
    let _ = app.emit("waypoint://passthrough-globally-off", should_show);
}
```

- [ ] **Step 2: ListWindow 加 listener（只處理 hide）**

ListWindow.svelte 既有 `let unlistenDeleted: ... = null;` 之後加：

```ts
let unlistenPassthroughOn: (() => void) | null = null;
```

`onMount` 內加：

```ts
unlistenPassthroughOn = await listen("waypoint://passthrough-globally-on", async () => {
  await windowsApi.hideWindow("list");
});
```

`onDestroy` 加 `unlistenPassthroughOn?.();`。

退穿透時 Rust 端直接 `open_list_window`，前端不需處理 globally-off 事件。

- [ ] **Step 3: cargo test + npm test**

```bash
cd src-tauri && cargo test --lib commands::passthrough_cmd
cd .. && npm test
```

Expected: 全綠

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/passthrough_cmd.rs src/windows/ListWindow.svelte
git commit -m "feat(passthrough): list auto hide on, restore on off via Rust"
```

---

## Task 5: e2e spec — passthrough chrome round-trip

**Files:**
- Create: `waypoint/e2e/specs/passthrough-chrome.spec.js`

- [ ] **Step 1: Write the test**

```js
// waypoint/e2e/specs/passthrough-chrome.spec.js
import assert from "node:assert/strict";

let listHandle;

async function waitTauriReady() {
  await browser.waitUntil(
    async () => browser.execute(
      () => typeof window.__TAURI_INTERNALS__?.invoke === "function"
    ),
    { timeout: 15_000 },
  );
}

async function invokeCmd(cmd, args = {}) {
  return browser.executeAsync(
    (c, a, done) => {
      window.__TAURI_INTERNALS__.invoke(c, a)
        .then((r) => done({ ok: true, value: r }))
        .catch((e) => done({ ok: false, error: String(e) }));
    },
    cmd,
    args,
  );
}

async function switchToNewWindow(prev) {
  await browser.waitUntil(
    async () => (await browser.getWindowHandles()).length > prev.length,
    { timeout: 10_000 },
  );
  const handles = await browser.getWindowHandles();
  const newHandle = handles.find((h) => !prev.includes(h));
  await browser.switchToWindow(newHandle);
}

describe("穿透模式 chrome 隱藏", () => {
  before(async () => {
    await browser.waitUntil(
      async () => (await browser.getPageSource()).includes("WAYPOINT"),
      { timeout: 20_000 },
    );
    await waitTauriReady();
    listHandle = (await browser.getWindowHandles())[0];
  });

  it("toggle on：note titlebar 隱藏；toggle off：還原", async () => {
    await browser.switchToWindow(listHandle);
    const created = await invokeCmd("create_note", { contextId: null, title: "PT-Chrome" });
    assert.equal(created.ok, true);
    const noteId = created.value.id;

    const before = await browser.getWindowHandles();
    await invokeCmd("cmd_open_note_window", { noteId, contextId: null });
    await switchToNewWindow(before);

    // 在 note webview：driving-titlebar 一開始可見
    let titlebarCount = await browser.execute(
      () => document.querySelectorAll('.draggable-titlebar').length,
    );
    assert.equal(titlebarCount, 1, "進穿透前 titlebar 應存在");

    // 從 list 觸發 global toggle on
    await browser.switchToWindow(listHandle);
    await invokeCmd("cmd_toggle_passthrough_global", {});
    await browser.pause(300);

    // 切回 note webview，driving-titlebar 應已不存在
    const handles = await browser.getWindowHandles();
    const noteHandle = handles.find((h) => h !== listHandle);
    await browser.switchToWindow(noteHandle);
    titlebarCount = await browser.execute(
      () => document.querySelectorAll('.draggable-titlebar').length,
    );
    assert.equal(titlebarCount, 0, "穿透 on 後 titlebar 應消失");

    // toggle off → titlebar 還原
    await browser.switchToWindow(listHandle);
    await invokeCmd("cmd_toggle_passthrough_global", {});
    await browser.pause(300);

    await browser.switchToWindow(noteHandle);
    titlebarCount = await browser.execute(
      () => document.querySelectorAll('.draggable-titlebar').length,
    );
    assert.equal(titlebarCount, 1, "穿透 off 後 titlebar 應還原");

    // cleanup
    await browser.switchToWindow(listHandle);
    await invokeCmd("cmd_close_note_window", { noteId });
    await invokeCmd("delete_note", { contextId: null, noteId });
  });
});
```

- [ ] **Step 2: Run via act**

```bash
cd /data/games-note-AIgen && export DOCKER_HOST=tcp://host.docker.internal:2375 && act -j e2e-linux > /tmp/act-planc.log 2>&1
grep -E "Spec Files|passthrough-chrome|failing" /tmp/act-planc.log
```

Expected: 綠

- [ ] **Step 3: Commit**

```bash
git add waypoint/e2e/specs/passthrough-chrome.spec.js
git commit -m "test(e2e): passthrough chrome hide/restore round-trip"
```

---

## Task 6: 全套驗證

- [ ] **Step 1: cargo + npm + render + act**

```bash
cd waypoint/src-tauri && cargo test
cd .. && npm test && npm run build && npm run test:render
cd /data/games-note-AIgen && export DOCKER_HOST=tcp://host.docker.internal:2375 && act -j e2e-linux > /tmp/act-planc-final.log 2>&1
grep -E "Spec Files|failing" /tmp/act-planc-final.log
```

Expected: 全綠

- [ ] **Step 2: Push**

```bash
git push origin master:refs/heads/dev/main
```

## Acceptance Criteria

- [ ] 穿透開：note titlebar 不渲染；title input 仍在；list 自我 hide
- [ ] 穿透關：titlebar 還原；list 視 snapshot 恢復
- [ ] 全套測試綠燈
