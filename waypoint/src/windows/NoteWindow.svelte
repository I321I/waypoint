<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { emit, listen } from "@tauri-apps/api/event";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Editor from "./note/Editor.svelte";
  import ContextMenu from "./note/ContextMenu.svelte";
  import TitlebarOpacitySlider from "./note/TitlebarOpacitySlider.svelte";
  import DraggableTitlebar from "./DraggableTitlebar.svelte";
  import GlobeIcon from "./icons/GlobeIcon.svelte";
  import { notes as notesApi, passthrough as passthroughApi, windows as windowsApi, config as configApi } from "../lib/api";
  import type { Note } from "../lib/types";
  import { parseTitleContent, joinTitleContent } from "../lib/noteFormat";

  export let noteId: string;
  export let contextId: string | null;

  let note: Note | null = null;
  let title: string = "";
  let body: string = "";
  let lastEmittedTitle: string = "";
  let editorRef: Editor;
  let menuOpen = false;
  let menuX = 0;
  let menuY = 0;
  let saveTimeout: ReturnType<typeof setTimeout>;
  let lastSelectionPos: number | null = null;
  let passthrough = false;
  let transparentIncludesText = true;
  let unlistenPassthrough: UnlistenFn | null = null;
  let unlistenRenamedFromList: UnlistenFn | null = null;
  let unlistenFlush: UnlistenFn | null = null;
  let unlistenDeleted: UnlistenFn | null = null;
  let unlistenConfigChanged: UnlistenFn | null = null;
  let unlistenMove: UnlistenFn | null = null;
  let unlistenResize: UnlistenFn | null = null;
  let unlistenFocusEnd: UnlistenFn | null = null;
  let unlistenWinFocus: UnlistenFn | null = null;
  let geometrySaveTimer: ReturnType<typeof setTimeout> | null = null;
  let saveGeometry: () => Promise<void> = async () => {};

  onMount(async () => {
    // 同步加上 class（不等 await），避免閃爍
    document.body.classList.add('note-view');
    note = await notesApi.read(contextId, noteId);
    if (note) {
      passthrough = note.settings.passthrough ?? false;
      const parsed = parseTitleContent(note.content);
      title = parsed.title || note.title || "";
      body = parsed.body;
      lastEmittedTitle = title;
    }
    transparentIncludesText = await configApi.getTransparentIncludesText().catch(() => true);
    unlistenConfigChanged = await listen("waypoint://config-changed", async () => {
      transparentIncludesText = await configApi.getTransparentIncludesText().catch(() => true);
    }).catch(() => null);
    unlistenPassthrough = await listen<[string, boolean]>("waypoint://passthrough-changed", (event) => {
      const [label, on] = event.payload;
      if (label === `note-${noteId}`) {
        passthrough = on;
      }
    }).catch(() => null);
    // 列表右鍵改名 -> 更新本視窗 title（不回送 title-changed 避免 echo loop）
    unlistenRenamedFromList = await listen<{ noteId: string; contextId: string | null; title: string }>(
      "waypoint://note-renamed-from-list",
      (event) => {
        if (event.payload.noteId !== noteId) return;
        title = event.payload.title;
        lastEmittedTitle = title;
        if (note) note = { ...note, title };
      }
    ).catch(() => null);
    // 結束 app 時主程序廣播 flush -> 立即 flushPendingSave
    unlistenFlush = await listen("waypoint://flush-and-save-now", async () => {
      await flushPendingSave();
    }).catch(() => null);
    // 筆記被刪除時（從列表或其他視窗）-> 強制關閉，不 flush/saveContent（檔案已不存在）
    unlistenDeleted = await listen<{ noteId: string; contextId: string | null }>(
      "waypoint://note-deleted",
      async (event) => {
        if (event.payload.noteId !== noteId) return;
        if ((event.payload.contextId ?? null) !== (contextId ?? null)) return;
        await emit("note-closed", { noteId, contextId, isGlobal: contextId === null });
        await windowsApi.closeNote(noteId);
      }
    ).catch(() => null);
    // 不註冊 onCloseRequested：webview2 + tauri v2 下任何 JS 端攔截都會讓 close 不執行。
    // Alt+F4 的 session 記帳改由 Rust on_window_event 處理（見 lib.rs）。

    // 幾何即時記憶：debounce 500ms 後寫 settings.windowBounds（item 7）
    const win = getCurrentWindow();
    saveGeometry = async () => {
      if (!note) return;
      try {
        const pos = await win.outerPosition();
        const size = await win.outerSize();
        const bounds = { x: pos.x, y: pos.y, width: size.width, height: size.height };
        const next = { ...note.settings, windowBounds: bounds };
        note = { ...note, settings: next };
        await notesApi.saveSettings(contextId, noteId, next);
      } catch {
        /* 視窗剛關掉等情境，吞 */
      }
    };
    const scheduleGeometrySave = () => {
      if (geometrySaveTimer) clearTimeout(geometrySaveTimer);
      geometrySaveTimer = setTimeout(saveGeometry, 500);
    };
    unlistenMove = await win.onMoved(scheduleGeometrySave).catch(() => null);
    unlistenResize = await win.onResized(scheduleGeometrySave).catch(() => null);

    // Rust 端建構 webview 時 visible(false)，等 HTML/CSS ready 才顯示，避免 Linux
    // WebKitGTK 透明 webview 在 paint 完成前露 OS 預設視窗 bg 的深色閃爍。
    win.show().catch(() => {});
    win.setAlwaysOnTop(true).catch(() => {});
    win.setFocus().catch(() => {});

    // 穿透模式關閉時，Rust 端會 emit 帶 last-edited noteId；比對中就 focus 並把游標放回
    // 進入穿透前的最後位置（user 進入穿透前在 3 和 4 中間點過，回來就回到 3 和 4 中間）。
    // 若沒記錄過位置（剛開沒互動過）就 fallback 到末。
    unlistenFocusEnd = await listen<string>("waypoint://focus-and-cursor-end", (event) => {
      if (event.payload !== noteId) return;
      const ed = editorRef?.getEditor();
      if (!ed) return;
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const focus = (ed as any).commands?.focus;
      if (lastSelectionPos != null) {
        focus?.(lastSelectionPos);
      } else {
        focus?.("end");
      }
    }).catch(() => null);

    // 視窗取得焦點時也標記為 last-edited（user 用滑鼠點到的筆記應該被當成「最後互動」，
    // 不只是有打字的才算；解穿透時的自動 focus 對象才符合預期）
    unlistenWinFocus = await win.onFocusChanged(({ payload: focused }) => {
      if (focused) {
        passthroughApi.markNoteEdited(noteId).catch(() => {});
      }
    }).catch(() => null);
  });

  onDestroy(() => {
    unlistenConfigChanged?.();
    unlistenPassthrough?.();
    unlistenRenamedFromList?.();
    unlistenFlush?.();
    unlistenDeleted?.();
    unlistenMove?.();
    unlistenResize?.();
    unlistenFocusEnd?.();
    unlistenWinFocus?.();
    if (geometrySaveTimer) clearTimeout(geometrySaveTimer);
  });

  async function handleDotClick() {
    await passthroughApi.toggleGlobal().catch(() => {});
  }

  function scheduleSave() {
    if (!note) return;
    clearTimeout(saveTimeout);
    saveTimeout = setTimeout(async () => {
      const merged = joinTitleContent(title, body);
      await notesApi.saveContent(contextId, noteId, merged);
      if (title !== lastEmittedTitle) {
        lastEmittedTitle = title;
        await emit("waypoint://note-title-changed", { noteId, contextId, title });
      }
    }, 100);
  }

  function handleTitleInput(e: Event) {
    title = (e.target as HTMLInputElement).value;
    scheduleSave();
    handleTitleEdit();
  }

  function handleContentUpdate(e: CustomEvent<{ markdown: string }>) {
    body = e.detail.markdown;
    scheduleSave();
    // 標記本 note 為當前 context 最後編輯 → 穿透模式關閉時自動 focus 這裡
    passthroughApi.markNoteEdited(noteId).catch(() => {});
  }

  function handleSelectionUpdate(e: CustomEvent<{ from: number; to: number }>) {
    // 記住游標位置：穿透模式關閉時，focus-and-cursor-end 會用它把游標放回原處
    lastSelectionPos = e.detail.from;
    // selection 變化也算「互動」 → 標記為 last-edited，含「在某位置點一下」這種沒打字的場景
    passthroughApi.markNoteEdited(noteId).catch(() => {});
  }

  function handleTitleEdit() {
    // title 改動也算編輯 → 標記 last-edited
    passthroughApi.markNoteEdited(noteId).catch(() => {});
  }

  function handleContextMenu(e: MouseEvent) {
    menuX = e.clientX;
    menuY = e.clientY;
    menuOpen = true;
  }

  async function flushPendingSave() {
    clearTimeout(saveTimeout);
    if (editorRef) {
      const ed = editorRef.getEditor();
      // tiptap v3：editor.getMarkdown()（舊版 storage.markdown.getMarkdown 在 v3 是 undefined，會吞掉用戶輸入）
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const md = (ed as any)?.getMarkdown?.() ?? null;
      if (md !== null) body = md;
    }
    const merged = joinTitleContent(title, body);
    await notesApi.saveContent(contextId, noteId, merged);
    // 一併 flush 幾何（debounce 中也立刻寫）
    if (geometrySaveTimer) {
      clearTimeout(geometrySaveTimer);
      geometrySaveTimer = null;
    }
    await saveGeometry();
  }

  async function handleClose() {
    await flushPendingSave();
    await emit("note-closed", { noteId, contextId, isGlobal: contextId === null });
    await windowsApi.closeNote(noteId);
  }

  async function handleMaximize() {
    await windowsApi.toggleMaximize(`note-${noteId}`);
  }

  async function handleCollapseAll() {
    await flushPendingSave();
    await emit("waypoint://collapse-all-requested");
  }

</script>

{#if note}
  <div
    class="note-window"
    class:translucent-text={transparentIncludesText}
    style="--note-bg-alpha: {note.settings.opacity}"
  >
    {#if !passthrough}
      <DraggableTitlebar label={`note-${noteId}`}>
        <span class="note-title" data-tauri-drag-region>
          {title || "Untitled"}
          {#if contextId === null}<GlobeIcon size={12} />{/if}
        </span>
        <TitlebarOpacitySlider
          opacity={note.settings.opacity}
          on:change={async (e) => {
            if (!note) return;
            const next = { ...note.settings, opacity: e.detail };
            note = { ...note, settings: next };
            await notesApi.saveSettings(contextId, noteId, next);
          }}
        />
        <div class="titlebar-buttons">
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
    {/if}

    <div class="title-row">
      <input
        class="title-input"
        type="text"
        placeholder="標題"
        value={title}
        on:input={handleTitleInput}
      />
    </div>

    <div class="editor-area" on:contextmenu|preventDefault={handleContextMenu}>
      <Editor
        bind:this={editorRef}
        content={body}
        fontSize={note.settings.fontSize}
        on:update={handleContentUpdate}
        on:selection={handleSelectionUpdate}
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

  </div>
{/if}

<style>
  :global(body.note-view) {
    background: transparent !important;
  }

  .note-window {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background-color: rgba(30, 30, 30, var(--note-bg-alpha, 1));
    border: 1px solid var(--border);
  }
  .note-window.translucent-text {
    opacity: var(--note-bg-alpha, 1);
  }
  .note-title {
    font-size: 12px;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    /* 與 .opacity-bar 一起 share titlebar 剩餘寬度，但 title 有 2x 權重；
       太短 title（如 "123"）不會被 slider 擠到只剩 50%。 */
    flex: 2 1 auto;
    min-width: 0;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    pointer-events: none;  /* 讓 mousedown 打到 .titlebar 本體而非 span */
  }
  .titlebar-buttons { display: flex; gap: 6px; flex-shrink: 0; align-items: center; }
  .passthrough-dot {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    border: none;
    padding: 0;
    cursor: pointer;
    flex-shrink: 0;
  }
  .dot-on  { background: #5cb85c; }
  .dot-off { background: #ffb454; box-shadow: 0 0 6px #ffb454; }
  .title-row {
    padding: 8px 12px 4px;
    background: transparent;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;  /* 防 editor-area flex: 1 把 title-row 擠扁 */
  }
  .title-input {
    width: 100%;
    background: transparent;
    border: none;
    color: var(--text-primary);
    font-size: 16px;
    font-weight: 600;
    padding: 4px 0;
    /* Linux WebKitGTK 上 input 不認 unitless line-height、預設 box 高 = font-size，
       導致 baseline 跑出 box 只剩 descender 可見（v0.2.7~0.2.10 user 回報「title 只剩下 1/5」）。
       顯式 line-height + min-height 強制撐開到正常行高。 */
    line-height: 1.4;
    min-height: 28px;
    box-sizing: content-box;
  }
  .title-input:focus { outline: none; border-bottom: 1px solid var(--accent); }
  .editor-area { display: flex; flex: 1; overflow: hidden; }
  .note-window :global(.editor-wrap) { scrollbar-width: none; }
  .note-window :global(.editor-wrap)::-webkit-scrollbar { width: 0; height: 0; display: none; }
</style>
