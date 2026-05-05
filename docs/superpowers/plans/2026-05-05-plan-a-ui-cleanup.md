# Plan A — 純 UI 清理 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 移除筆記 markdown toolbar 與 SettingsPanel、改用右鍵浮動選單；改 titlebar 顯示規則（全域加 globe icon，區域只名稱）；隱藏筆記視窗 scrollbar。

**Architecture:** 全部前端改動，無 Rust 變更。刪 `Toolbar.svelte`、`SettingsPanel.svelte`、`TitlebarOpacitySlider.svelte` 不動。新增 `note/ContextMenu.svelte` 與 `icons/GlobeIcon.svelte`。NoteWindow 模板簡化、加 CSS 隱藏 scrollbar。

**Tech Stack:** Svelte 4、tiptap、Vitest、Playwright（render tests）、WDIO（e2e wdio specs）。

**Dependencies:** 無前置 plan。

---

## File Structure

| File | Status | Responsibility |
|---|---|---|
| `src/windows/note/ContextMenu.svelte` | Create | 右鍵浮動選單，4 項固定（複製、貼上、字體大小、刪除） |
| `src/windows/icons/GlobeIcon.svelte` | Create | inline outlined globe SVG |
| `src/windows/note/Toolbar.svelte` | Delete | 已不用 |
| `src/windows/note/SettingsPanel.svelte` | Delete | 已不用 |
| `src/windows/NoteWindow.svelte` | Modify | 拿掉 `<Toolbar>`/`<SettingsPanel>`、改 titlebar 顯示規則、加 contextmenu listener、加 scrollbar 隱藏 CSS |
| `src/windows/note/ContextMenu.render.test.pw.ts` | Create | render：4 項可見、stepper 改值會 dispatch、刪除走 ConfirmDialog |
| `src/windows/NoteWindow.titlebar.render.test.pw.ts` | Modify | 全域筆記出現 GlobeIcon、區域筆記不出現「-」字樣 |
| `src/windows/NoteWindow.scrollbar.render.test.pw.ts` | Create | scrollbar 寬度為 0 |
| `src/lib/api.ts` | No change | API 已涵蓋 |

---

## Task 1: 建立 GlobeIcon 元件

**Files:**
- Create: `src/windows/icons/GlobeIcon.svelte`
- Create: `src/windows/icons/GlobeIcon.render.test.pw.ts`

- [ ] **Step 1: Write the failing test**

掛載單一元件麻煩，直接走 NoteWindow 既有 Tauri mock pattern：

```ts
// src/windows/icons/GlobeIcon.render.test.pw.ts
import { test, expect } from '@playwright/test';

test('GlobeIcon 在 NoteWindow（global）會渲染', async ({ page }) => {
  await page.addInitScript(() => {
    (window as any).__TAURI_INTERNALS__ = {
      metadata: { currentWindow: { label: 'note-x' }, currentWebview: { label: 'note-x', windowLabel: 'note-x' } },
      invoke: (cmd: string) => {
        if (cmd === 'read_note') return Promise.resolve({
          id: 'x', contextId: null, title: 'T', content: '',
          settings: { fontSize: 14, opacity: 1, hotkey: null, windowBounds: null, passthrough: false },
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
  await expect(page.locator('.note-title svg[data-globe-icon]')).toBeVisible();
});
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd /data/games-note-AIgen/waypoint && npm run build && npx playwright test src/windows/icons/GlobeIcon.render.test.pw.ts
```

Expected: FAIL — selector `svg[data-globe-icon]` not found（NoteWindow 還沒掛 GlobeIcon）

- [ ] **Step 3: Implement GlobeIcon.svelte**

```svelte
<!-- src/windows/icons/GlobeIcon.svelte -->
<script lang="ts">
  export let size: number = 14;
</script>

<svg
  data-globe-icon
  xmlns="http://www.w3.org/2000/svg"
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="1.6"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  <circle cx="12" cy="12" r="9" />
  <path d="M3 12h18" />
  <path d="M12 3a14 14 0 0 1 0 18a14 14 0 0 1 0-18z" />
</svg>

<style>
  svg { color: var(--text-secondary); flex-shrink: 0; vertical-align: middle; }
</style>
```

- [ ] **Step 4: 暫不跑 test，等 Task 2 把 NoteWindow 接上 GlobeIcon 才會通過**

- [ ] **Step 5: Commit**

```bash
git add src/windows/icons/GlobeIcon.svelte src/windows/icons/GlobeIcon.render.test.pw.ts
git commit -m "feat(ui): GlobeIcon component for global note titlebar"
```

---

## Task 2: NoteWindow titlebar 規則改寫

**Files:**
- Modify: `src/windows/NoteWindow.svelte:159`
- Modify: `src/windows/NoteWindow.titlebar.render.test.pw.ts`

- [ ] **Step 1: 改 NoteWindow 模板**

把 NoteWindow.svelte 第 159 行：

```svelte
<span class="note-title" data-tauri-drag-region>{(title || "Untitled") + "-" + (contextId ?? "Global")}</span>
```

改成：

```svelte
<span class="note-title" data-tauri-drag-region>
  {title || "Untitled"}
  {#if contextId === null}<GlobeIcon size={12} />{/if}
</span>
```

並在 script 區塊頂部加：

```ts
import GlobeIcon from "./icons/GlobeIcon.svelte";
```

- [ ] **Step 2: 改 titlebar render test**

替換 `src/windows/NoteWindow.titlebar.render.test.pw.ts` 內容：

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
        if (cmd === 'plugin:event|listen') return Promise.resolve(0);
        return Promise.resolve(null);
      },
      transformCallback: () => 0, unregisterCallback: () => {}, convertFileSrc: (s: string) => s,
    };
  }, { ctx: contextId, t: title });
}

test('全域筆記 titlebar 顯示標題 + GlobeIcon', async ({ page }) => {
  await mockNote(page, null, '1122');
  await page.goto('http://localhost:4173/#view=note&noteId=x');
  await page.waitForLoadState('networkidle');
  const title = page.locator('.note-title');
  await expect(title).toContainText('1122');
  await expect(title.locator('svg[data-globe-icon]')).toBeVisible();
});

test('區域筆記 titlebar 只顯示標題，無 GlobeIcon、無 dash', async ({ page }) => {
  await mockNote(page, 'edge', '我是誰');
  await page.goto('http://localhost:4173/#view=note&noteId=x&contextId=edge');
  await page.waitForLoadState('networkidle');
  const title = page.locator('.note-title');
  await expect(title).toHaveText('我是誰');
  await expect(title.locator('svg[data-globe-icon]')).toHaveCount(0);
});

test('NoteWindow 不再顯示 .statusbar', async ({ page }) => {
  await mockNote(page, null, 'T');
  await page.goto('http://localhost:4173/#view=note&noteId=x');
  await page.waitForLoadState('networkidle');
  await expect(page.locator('.statusbar')).toHaveCount(0);
});
```

- [ ] **Step 3: 跑 test 驗證綠**

```bash
cd /data/games-note-AIgen/waypoint && npm run build && npx playwright test src/windows/NoteWindow.titlebar.render.test.pw.ts src/windows/icons/GlobeIcon.render.test.pw.ts
```

Expected: 全綠

- [ ] **Step 4: Commit**

```bash
git add src/windows/NoteWindow.svelte src/windows/NoteWindow.titlebar.render.test.pw.ts
git commit -m "feat(ui): titlebar shows title + GlobeIcon for global, name only for context"
```

---

## Task 3: 建立 ContextMenu 元件

**Files:**
- Create: `src/windows/note/ContextMenu.svelte`
- Create: `src/windows/note/ContextMenu.render.test.pw.ts`

- [ ] **Step 1: Write the failing test**

```ts
// src/windows/note/ContextMenu.render.test.pw.ts
import { test, expect, type Page } from '@playwright/test';

async function mockNote(page: Page) {
  await page.addInitScript(() => {
    (window as any).__TAURI_INTERNALS__ = {
      metadata: { currentWindow: { label: 'note-x' }, currentWebview: { label: 'note-x', windowLabel: 'note-x' } },
      invoke: (cmd: string, args: any) => {
        if (cmd === 'read_note') return Promise.resolve({
          id: 'x', contextId: null, title: 'T', content: 'hello',
          settings: { fontSize: 14, opacity: 1, hotkey: null, windowBounds: null, passthrough: false },
        });
        if (cmd === 'get_transparent_includes_text') return Promise.resolve(true);
        if (cmd === 'plugin:event|listen') return Promise.resolve(0);
        if (cmd === 'save_note_settings') {
          (window as any).__lastSavedFontSize = args.settings.fontSize;
          return Promise.resolve(null);
        }
        if (cmd === 'delete_note') {
          (window as any).__deleteCalled = true;
          return Promise.resolve(null);
        }
        return Promise.resolve(null);
      },
      transformCallback: () => 0, unregisterCallback: () => {}, convertFileSrc: (s: string) => s,
    };
  });
}

test('右鍵 editor 區會顯示 ContextMenu，含 4 項', async ({ page }) => {
  await mockNote(page);
  await page.goto('http://localhost:4173/#view=note&noteId=x');
  await page.waitForLoadState('networkidle');
  await page.locator('.editor-area').click({ button: 'right' });
  await expect(page.locator('.context-menu')).toBeVisible();
  await expect(page.locator('.context-menu [data-mi="copy"]')).toBeVisible();
  await expect(page.locator('.context-menu [data-mi="paste"]')).toBeVisible();
  await expect(page.locator('.context-menu [data-mi="font-size"]')).toBeVisible();
  await expect(page.locator('.context-menu [data-mi="delete"]')).toBeVisible();
});

test('字體大小 stepper +/- 會 dispatch save_note_settings', async ({ page }) => {
  await mockNote(page);
  await page.goto('http://localhost:4173/#view=note&noteId=x');
  await page.waitForLoadState('networkidle');
  await page.locator('.editor-area').click({ button: 'right' });
  await page.locator('.context-menu [data-mi="font-size"] [data-act="inc"]').click();
  await page.waitForTimeout(50);
  expect(await page.evaluate(() => (window as any).__lastSavedFontSize)).toBe(15);
});

test('刪除此筆記會走 ConfirmDialog → delete_note', async ({ page }) => {
  await mockNote(page);
  await page.goto('http://localhost:4173/#view=note&noteId=x');
  await page.waitForLoadState('networkidle');
  await page.locator('.editor-area').click({ button: 'right' });
  await page.locator('.context-menu [data-mi="delete"]').click();
  await expect(page.locator('.dialog')).toBeVisible();
  await page.locator('.dialog button.danger').click();
  await page.waitForTimeout(50);
  expect(await page.evaluate(() => (window as any).__deleteCalled)).toBe(true);
});

test('Escape 關閉 ContextMenu', async ({ page }) => {
  await mockNote(page);
  await page.goto('http://localhost:4173/#view=note&noteId=x');
  await page.waitForLoadState('networkidle');
  await page.locator('.editor-area').click({ button: 'right' });
  await expect(page.locator('.context-menu')).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(page.locator('.context-menu')).toHaveCount(0);
});
```

- [ ] **Step 2: Run test to verify it fails**

```bash
npm run build && npx playwright test src/windows/note/ContextMenu.render.test.pw.ts
```

Expected: FAIL — `.context-menu` not found

- [ ] **Step 3: Implement ContextMenu.svelte**

```svelte
<!-- src/windows/note/ContextMenu.svelte -->
<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { onMount, onDestroy } from "svelte";
  import type { Editor } from "@tiptap/core";
  import ConfirmDialog from "../ConfirmDialog.svelte";
  import { notes as notesApi } from "../../lib/api";

  export let editor: Editor | null;
  export let fontSize: number;
  export let noteId: string;
  export let contextId: string | null;
  export let x: number;
  export let y: number;

  const dispatch = createEventDispatcher<{
    close: void;
    'font-size-change': number;
  }>();

  let confirmingDelete = false;

  function close() { dispatch('close'); }

  async function copyAction() {
    if (!editor) return;
    // tiptap selection → markdown：用 editor.state.selection 抓出 slice 的 markdown
    const { from, to } = editor.state.selection;
    if (from === to) { close(); return; }
    // editor.storage.markdown? v3 用 getMarkdown 全文。簡化：複製選取的純文字 + markdown via temporary
    const text = editor.state.doc.textBetween(from, to, '\n');
    try { await navigator.clipboard.writeText(text); } catch {}
    close();
  }

  async function pasteAction() {
    if (!editor) return;
    try {
      const text = await navigator.clipboard.readText();
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (editor.commands as any).insertContent(text);
    } catch {}
    close();
  }

  function fontStep(delta: number) {
    const next = Math.max(8, Math.min(32, fontSize + delta));
    fontSize = next;
    dispatch('font-size-change', next);
  }

  async function doDelete() {
    confirmingDelete = false;
    await notesApi.delete(contextId, noteId);
    close();
  }

  function handleKey(e: KeyboardEvent) {
    if (e.key === 'Escape') close();
  }

  function handleClickOutside(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (!target.closest('.context-menu') && !target.closest('.dialog')) close();
  }

  onMount(() => {
    document.addEventListener('keydown', handleKey);
    document.addEventListener('mousedown', handleClickOutside);
  });
  onDestroy(() => {
    document.removeEventListener('keydown', handleKey);
    document.removeEventListener('mousedown', handleClickOutside);
  });
</script>

<div class="context-menu" style="left: {x}px; top: {y}px;">
  <button data-mi="copy" on:click={copyAction}>複製</button>
  <button data-mi="paste" on:click={pasteAction}>貼上</button>
  <div data-mi="font-size" class="stepper-row">
    <span>字體大小</span>
    <button data-act="dec" on:click={() => fontStep(-1)}>−</button>
    <span class="num">{fontSize}</span>
    <button data-act="inc" on:click={() => fontStep(1)}>+</button>
  </div>
  <button data-mi="delete" class="danger" on:click={() => confirmingDelete = true}>刪除此筆記</button>
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
  .context-menu {
    position: fixed;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 4px;
    box-shadow: 0 2px 8px rgba(0,0,0,.4);
    padding: 4px 0;
    min-width: 160px;
    z-index: 1100;
    display: flex;
    flex-direction: column;
    font-size: 12px;
  }
  .context-menu button {
    text-align: left;
    background: transparent;
    border: none;
    color: var(--text-primary);
    padding: 6px 12px;
    cursor: pointer;
  }
  .context-menu button:hover { background: var(--bg-tertiary); }
  .context-menu button.danger { color: var(--accent-danger, #c0392b); }
  .stepper-row {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 12px;
    border-top: 1px solid var(--border);
    border-bottom: 1px solid var(--border);
  }
  .stepper-row span { flex: 1; }
  .stepper-row .num { flex: 0 0 28px; text-align: center; }
  .stepper-row button { padding: 0 6px; font-size: 12px; }
</style>
```

- [ ] **Step 4: Wire up in NoteWindow.svelte**

把 NoteWindow.svelte 內 `<Toolbar>` 與 `<SettingsPanel>` 段落整段刪除，editor-area 改成：

```svelte
<div class="editor-area" on:contextmenu|preventDefault={handleContextMenu}>
  <Editor
    bind:this={editorRef}
    content={body}
    fontSize={note.settings.fontSize}
    on:update={handleContentUpdate}
  />
  {#if menuOpen}
    <ContextMenu
      editor={editorRef?.getEditor() ?? null}
      fontSize={note.settings.fontSize}
      {noteId}
      {contextId}
      x={menuX}
      y={menuY}
      on:close={() => menuOpen = false}
      on:font-size-change={async (e) => {
        if (!note) return;
        const next = { ...note.settings, fontSize: e.detail };
        note = { ...note, settings: next };
        await notesApi.saveSettings(contextId, noteId, next);
      }}
    />
  {/if}
</div>
```

並在 script 區加：

```ts
import ContextMenu from "./note/ContextMenu.svelte";

let menuOpen = false;
let menuX = 0;
let menuY = 0;
function handleContextMenu(e: MouseEvent) {
  menuX = e.clientX;
  menuY = e.clientY;
  menuOpen = true;
}
```

並刪除 imports：`Toolbar`、`SettingsPanel`、`settingsOpen` 變數。

- [ ] **Step 5: 刪除舊檔**

```bash
rm src/windows/note/Toolbar.svelte src/windows/note/SettingsPanel.svelte src/windows/note/SettingsPanel.delete.render.test.pw.ts
```

`SettingsPanel.delete.render.test.pw.ts` 的測試意圖搬到 `ContextMenu.render.test.pw.ts` 的「刪除此筆記會走 ConfirmDialog → delete_note」案例，已涵蓋。

- [ ] **Step 6: Run all render tests**

```bash
npm run build && npm run test:render
```

Expected: 全綠（含新加的 ContextMenu 4 個 case + 既有更新的 titlebar）

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat(ui): replace toolbar/settings panel with right-click ContextMenu"
```

---

## Task 4: 隱藏筆記視窗 scrollbar

**Files:**
- Modify: `src/windows/NoteWindow.svelte` `<style>`
- Create: `src/windows/NoteWindow.scrollbar.render.test.pw.ts`

- [ ] **Step 1: Write the failing test**

```ts
// src/windows/NoteWindow.scrollbar.render.test.pw.ts
import { test, expect, type Page } from '@playwright/test';

async function mockNote(page: Page) {
  await page.addInitScript(() => {
    (window as any).__TAURI_INTERNALS__ = {
      metadata: { currentWindow: { label: 'note-x' }, currentWebview: { label: 'note-x', windowLabel: 'note-x' } },
      invoke: (cmd: string) => {
        if (cmd === 'read_note') return Promise.resolve({
          id: 'x', contextId: null, title: 'T',
          content: 'a\n'.repeat(200),
          settings: { fontSize: 14, opacity: 1, hotkey: null, windowBounds: null, passthrough: false },
        });
        if (cmd === 'get_transparent_includes_text') return Promise.resolve(true);
        if (cmd === 'plugin:event|listen') return Promise.resolve(0);
        return Promise.resolve(null);
      },
      transformCallback: () => 0, unregisterCallback: () => {}, convertFileSrc: (s: string) => s,
    };
  });
}

test('筆記 editor scrollbar 寬度為 0（webkit）', async ({ page }) => {
  await mockNote(page);
  await page.setViewportSize({ width: 320, height: 200 });
  await page.goto('http://localhost:4173/#view=note&noteId=x');
  await page.waitForLoadState('networkidle');
  const result = await page.evaluate(() => {
    const el = document.querySelector('.editor-wrap') as HTMLElement;
    return el ? el.offsetWidth - el.clientWidth : -1;
  });
  // editor-wrap 的 offsetWidth - clientWidth 應為 0（無可見 scrollbar）
  expect(result).toBeLessThanOrEqual(0);
});

test('筆記內容仍可滾動（scrollHeight > clientHeight）', async ({ page }) => {
  await mockNote(page);
  await page.setViewportSize({ width: 320, height: 200 });
  await page.goto('http://localhost:4173/#view=note&noteId=x');
  await page.waitForLoadState('networkidle');
  const overflowing = await page.evaluate(() => {
    const el = document.querySelector('.editor-wrap') as HTMLElement;
    return el ? el.scrollHeight > el.clientHeight : false;
  });
  expect(overflowing).toBe(true);
});
```

- [ ] **Step 2: Run test, verify it fails**

```bash
npx playwright test src/windows/NoteWindow.scrollbar.render.test.pw.ts
```

Expected: FAIL（webkit 預設 scrollbar 寬度 > 0）

- [ ] **Step 3: 加 CSS**

NoteWindow.svelte `<style>` 區末加：

```css
.note-window :global(.editor-wrap) {
  scrollbar-width: none;
}
.note-window :global(.editor-wrap)::-webkit-scrollbar {
  width: 0;
  height: 0;
  display: none;
}
```

- [ ] **Step 4: Run test，驗證綠**

```bash
npx playwright test src/windows/NoteWindow.scrollbar.render.test.pw.ts
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/windows/NoteWindow.svelte src/windows/NoteWindow.scrollbar.render.test.pw.ts
git commit -m "feat(ui): hide scrollbar on note editor (still scrollable)"
```

---

## Task 5: 全套驗證

- [ ] **Step 1: Vitest**

```bash
cd /data/games-note-AIgen/waypoint && npm test
```

Expected: 全綠（24 passed 或更多，因為刪了 Toolbar/SettingsPanel 的測試但加了 ContextMenu 的）

- [ ] **Step 2: 重 build + render tests**

```bash
npm run build && npm run test:render
```

Expected: 全綠（GlobeIcon, NoteWindow.titlebar, ContextMenu, NoteWindow.scrollbar 全綠；先前的 deleted/passthrough/transparent 仍綠）

- [ ] **Step 3: cargo test（驗證沒誤動 backend）**

```bash
cd src-tauri && cargo test
```

Expected: 68 passed 全綠

- [ ] **Step 4: act e2e-linux**

```bash
cd /data/games-note-AIgen && export DOCKER_HOST=tcp://host.docker.internal:2375 && act -j e2e-linux > /tmp/act-plana.log 2>&1
grep -E "Spec Files|failing" /tmp/act-plana.log
```

Expected: 13/13 全綠（既有 spec 中除了 SettingsPanel.delete render（已刪）外，note-deletion / altf4-close 等仍依靠舊行為的 spec 應仍可運作，因為 ContextMenu 的「刪除此筆記」走同一條 `delete_note` 路徑）

⚠️ 風險：原 `SettingsPanel.delete.render.test.pw.ts` 被刪。`note-deletion.spec.js` 第二個 case「從筆記設定面板刪除筆記 → 列表同步移除」名稱對應的場景已不存在，但實際上 spec 是切回 list 直接呼叫 `delete_note`（不依賴 SettingsPanel UI），所以仍可運作。需 act 驗證。

- [ ] **Step 5: Commit + push**

```bash
git push origin master:refs/heads/dev/main
```

## Acceptance Criteria

- [ ] NoteWindow 視覺上不再有 markdown toolbar
- [ ] 右鍵 editor 區出現 ContextMenu 含 4 項
- [ ] 字體大小 stepper 即時 save 並反映在 editor 字體
- [ ] 刪除走 ConfirmDialog
- [ ] 全域筆記 titlebar：標題 + globe icon
- [ ] 區域筆記 titlebar：只有標題，無 dash、無 context 字串
- [ ] 筆記內容 scrollbar 視覺上不可見、仍可滾動
- [ ] 所有測試（vitest + render + cargo + act e2e-linux）綠燈
