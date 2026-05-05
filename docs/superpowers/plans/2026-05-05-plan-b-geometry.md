# Plan B — 筆記幾何即時記憶 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 筆記視窗大小／位置在每次移動或 resize 時自動寫入 `note.settings.windowBounds`（debounce 500ms），下次開窗自動還原，不只 ✕ 關閉時保存。

**Architecture:** 前端 NoteWindow 註冊 `getCurrentWindow().onMoved/onResized` listener，收到事件 debounce 後呼叫 `notes.saveSettings`。Rust `hotkey::open_note_window` 開窗時讀 `note.settings.window_bounds` 套用 `.position(x,y).inner_size(w,h)`。

**Tech Stack:** Tauri v2 window event API、既有 NoteSettings.windowBounds（schema 已存在）。

**Dependencies:** 建議先完成 Plan A（避免衝突修同檔），但獨立可行。

---

## File Structure

| File | Status | Responsibility |
|---|---|---|
| `src/windows/NoteWindow.svelte` | Modify | onMount 加 onMoved/onResized debounced save |
| `src-tauri/src/hotkey/mod.rs` | Modify | `open_note_window` 開窗時套用 `note.settings.window_bounds` |
| `src/windows/NoteWindow.svelte` 或新 util | Modify | 加 debounce util |
| `e2e/specs/note-geometry.spec.js` | Create | 拖動 + resize 後關閉 → 重開斷言位置／大小 |

---

## Task 1: Rust open_note_window 套用 saved geometry

**Files:**
- Modify: `src-tauri/src/hotkey/mod.rs:127`

- [ ] **Step 1: 改 `open_note_window`**

把 hotkey/mod.rs 第 127–156 行的 `open_note_window`：

```rust
pub fn open_note_window(app: &AppHandle, note_id: &str, context_id: Option<&str>) -> tauri::Result<()> {
    let label = format!("note-{}", note_id);
    if let Ok(mut map) = app.state::<crate::state::AppState>().open_notes_context.lock() {
        map.insert(note_id.to_string(), context_id.map(|s| s.to_string()));
    }
    if let Some(win) = app.get_webview_window(&label) {
        win.show()?;
        win.set_focus()?;
        return Ok(());
    }
    let ctx_param = context_id.map(|c| format!("&contextId={}", c)).unwrap_or_default();
    let url = format!("/#view=note&noteId={}{}", note_id, ctx_param);
    let mut builder = WebviewWindowBuilder::new(app, &label, WebviewUrl::App(url.into()))
        .title("Waypoint Note")
        .inner_size(420.0, 600.0)
        .min_inner_size(300.0, 200.0)
        .resizable(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .transparent(true);

    // 讀 settings.window_bounds 還原大小/位置（沒有就用上方預設）
    if let Ok(note) = crate::storage::notes::read_note(context_id, note_id) {
        if let Some(b) = note.settings.window_bounds {
            builder = builder
                .position(b.x as f64, b.y as f64)
                .inner_size(b.width as f64, b.height as f64);
        }
    }

    builder.build()?;
    crate::taskbar::refresh_taskbar_visibility(app);
    Ok(())
}
```

- [ ] **Step 2: Cargo build 確認 compile**

```bash
cd /data/games-note-AIgen/waypoint/src-tauri && cargo build 2>&1 | tail -10
```

Expected: `Finished` 無 error

- [ ] **Step 3: Cargo test 既有 hotkey tests 仍綠**

```bash
cargo test --lib hotkey
```

Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/hotkey/mod.rs
git commit -m "feat(geometry): open_note_window 套用 saved window_bounds"
```

---

## Task 2: 前端 onMoved/onResized debounced save

**Files:**
- Modify: `src/windows/NoteWindow.svelte`

- [ ] **Step 1: 加入 debounce + listeners**

NoteWindow.svelte script 區頂部加：

```ts
import { getCurrentWindow } from "@tauri-apps/api/window";
```

在現有 `let unlistenDeleted: UnlistenFn | null = null;` 之後加：

```ts
let unlistenMove: UnlistenFn | null = null;
let unlistenResize: UnlistenFn | null = null;
let geometrySaveTimer: ReturnType<typeof setTimeout> | null = null;
```

`onMount` 內 `transparentIncludesText = ...` 之後加：

```ts
// 幾何（位置/大小）即時記憶：debounce 500ms 後寫入 settings.windowBounds
const win = getCurrentWindow();

const saveGeometry = async () => {
  if (!note) return;
  try {
    const pos = await win.outerPosition();
    const size = await win.outerSize();
    const bounds = { x: pos.x, y: pos.y, width: size.width, height: size.height };
    const next = { ...note.settings, windowBounds: bounds };
    note = { ...note, settings: next };
    await notesApi.saveSettings(contextId, noteId, next);
  } catch {
    /* 視窗剛被關掉等場景，吞 */
  }
};

const scheduleGeometrySave = () => {
  if (geometrySaveTimer) clearTimeout(geometrySaveTimer);
  geometrySaveTimer = setTimeout(saveGeometry, 500);
};

unlistenMove = await win.onMoved(scheduleGeometrySave).catch(() => null);
unlistenResize = await win.onResized(scheduleGeometrySave).catch(() => null);
```

`onDestroy` 內加：

```ts
unlistenMove?.();
unlistenResize?.();
if (geometrySaveTimer) clearTimeout(geometrySaveTimer);
```

- [ ] **Step 2: Vitest 確保現有測試仍綠**

```bash
cd /data/games-note-AIgen/waypoint && npm test
```

Expected: 全綠

- [ ] **Step 3: Commit**

```bash
git add src/windows/NoteWindow.svelte
git commit -m "feat(geometry): NoteWindow debounced save on move/resize"
```

---

## Task 3: e2e spec — 改大小/位置後關閉，重開保留

**Files:**
- Create: `waypoint/e2e/specs/note-geometry.spec.js`

- [ ] **Step 1: Write the failing test**

```js
// waypoint/e2e/specs/note-geometry.spec.js
import assert from "node:assert/strict";

let listHandle;

async function waitTauriReady() {
  await browser.waitUntil(
    async () => browser.execute(
      () => typeof window.__TAURI_INTERNALS__?.invoke === "function"
    ),
    { timeout: 15_000, timeoutMsg: "Tauri IPC 未就緒" },
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

async function switchToNewWindow(previousHandles) {
  await browser.waitUntil(
    async () => (await browser.getWindowHandles()).length > previousHandles.length,
    { timeout: 10_000, timeoutMsg: "新視窗沒出現" },
  );
  const handles = await browser.getWindowHandles();
  const newHandle = handles.find((h) => !previousHandles.includes(h));
  await browser.switchToWindow(newHandle);
}

async function switchToList() {
  await browser.switchToWindow(listHandle);
}

describe("筆記幾何即時記憶", () => {
  before(async () => {
    await browser.waitUntil(
      async () => (await browser.getPageSource()).includes("WAYPOINT"),
      { timeout: 20_000 },
    );
    await waitTauriReady();
    listHandle = (await browser.getWindowHandles())[0];
  });

  it("移動位置後關閉，重開應在新位置", async () => {
    await switchToList();
    const created = await invokeCmd("create_note", { contextId: null, title: "GeoTest" });
    assert.equal(created.ok, true, created.error);
    const noteId = created.value.id;

    const before = await browser.getWindowHandles();
    const opened = await invokeCmd("cmd_open_note_window", { noteId, contextId: null });
    assert.equal(opened.ok, true, opened.error);
    await switchToNewWindow(before);

    // 從 list 設位置（避免從 note webview 操作干擾）
    await switchToList();
    await invokeCmd("cmd_set_window_position", { label: `note-${noteId}`, x: 150, y: 220 });

    // 等 debounce 500ms + io
    await browser.pause(800);

    // 從 list 關掉視窗
    await invokeCmd("cmd_close_note_window", { noteId });
    await browser.waitUntil(
      async () => (await browser.getWindowHandles()).length === before.length,
      { timeout: 5_000 },
    );

    // 重開
    const before2 = await browser.getWindowHandles();
    await invokeCmd("cmd_open_note_window", { noteId, contextId: null });
    await switchToNewWindow(before2);

    await switchToList();
    const pos = await invokeCmd("cmd_get_window_position", { label: `note-${noteId}` });
    assert.equal(pos.ok, true, pos.error);
    const [x, y] = pos.value;
    assert.equal(x, 150, `重開後 x 應為 150，實際 ${x}`);
    assert.equal(y, 220, `重開後 y 應為 220，實際 ${y}`);

    // cleanup
    await invokeCmd("delete_note", { contextId: null, noteId });
  });
});
```

- [ ] **Step 2: Run via act**

```bash
cd /data/games-note-AIgen && export DOCKER_HOST=tcp://host.docker.internal:2375 && act -j e2e-linux > /tmp/act-planb.log 2>&1
grep -E "Spec Files|note-geometry|failing" /tmp/act-planb.log
```

Expected: spec 綠

- [ ] **Step 3: Commit**

```bash
git add waypoint/e2e/specs/note-geometry.spec.js
git commit -m "test(e2e): note geometry persists across close+reopen"
```

---

## Task 4: 全套驗證

- [ ] **Step 1: cargo + npm test + render**

```bash
cd waypoint/src-tauri && cargo test
cd .. && npm test && npm run build && npm run test:render
```

Expected: 全綠

- [ ] **Step 2: act e2e-linux**

```bash
cd /data/games-note-AIgen && export DOCKER_HOST=tcp://host.docker.internal:2375 && act -j e2e-linux > /tmp/act-planb-final.log 2>&1
grep -E "Spec Files|failing" /tmp/act-planb-final.log
```

Expected: 14/14（原 13 + note-geometry）綠燈

- [ ] **Step 3: Push**

```bash
git push origin master:refs/heads/dev/main
```

## Acceptance Criteria

- [ ] 移動筆記視窗後 500ms 內 settings.json 已更新 windowBounds
- [ ] resize 同樣
- [ ] 關閉並重開時 outerPosition / outerSize 還原
- [ ] act e2e-linux 全綠
