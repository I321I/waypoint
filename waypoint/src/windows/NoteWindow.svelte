<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { emit, listen } from "@tauri-apps/api/event";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import Editor from "./note/Editor.svelte";
  import Toolbar from "./note/Toolbar.svelte";
  import SettingsPanel from "./note/SettingsPanel.svelte";
  import TitlebarOpacitySlider from "./note/TitlebarOpacitySlider.svelte";
  import DraggableTitlebar from "./DraggableTitlebar.svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { notes as notesApi, passthrough as passthroughApi, windows as windowsApi, config as configApi } from "../lib/api";
  import type { Note, NoteSettings } from "../lib/types";
  import { parseTitleContent, joinTitleContent } from "../lib/noteFormat";

  export let noteId: string;
  export let contextId: string | null;

  let note: Note | null = null;
  let title: string = "";
  let body: string = "";
  let lastEmittedTitle: string = "";
  let settingsOpen = false;
  let editorRef: Editor;
  let saveTimeout: ReturnType<typeof setTimeout>;
  let passthrough = false;
  let transparentIncludesText = true;
  let unlistenPassthrough: UnlistenFn | null = null;
  let unlistenRenamedFromList: UnlistenFn | null = null;
  let unlistenFlush: UnlistenFn | null = null;
  let unlistenDeleted: UnlistenFn | null = null;
  let unlistenConfigChanged: UnlistenFn | null = null;
  let unlistenClose: (() => void) | null = null;
  let isProgrammaticClose = false;

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
        isProgrammaticClose = true;
        await emit("note-closed", { noteId, contextId, isGlobal: contextId === null });
        await getCurrentWindow().close();
      }
    ).catch(() => null);
    // 攔截 OS 關閉（Alt+F4 等）-> 走與 X 按鈕相同的 handleClose 流程
    unlistenClose = await getCurrentWindow().onCloseRequested(async (e) => {
      if (isProgrammaticClose) return; // 程式化 close 已發起，讓預設關閉通過（不 preventDefault）
      e.preventDefault();
      await handleClose();
    });
  });

  onDestroy(() => {
    unlistenConfigChanged?.();
    unlistenPassthrough?.();
    unlistenRenamedFromList?.();
    unlistenFlush?.();
    unlistenDeleted?.();
    unlistenClose?.();
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
  }

  function handleContentUpdate(e: CustomEvent<{ markdown: string }>) {
    body = e.detail.markdown;
    scheduleSave();
  }

  async function handleSettingsChange(e: CustomEvent<NoteSettings>) {
    if (!note) return;
    note = { ...note, settings: e.detail };
    await notesApi.saveSettings(contextId, noteId, e.detail);
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
  }

  async function handleClose() {
    if (isProgrammaticClose) return; // 重入防護
    isProgrammaticClose = true;
    await flushPendingSave();
    await emit("note-closed", { noteId, contextId, isGlobal: contextId === null });
    await getCurrentWindow().close();
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
    <DraggableTitlebar label={`note-${noteId}`}>
      <span class="note-title" data-tauri-drag-region>{(title || "Untitled") + "-" + (contextId ?? "Global")}</span>
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

    <div class="title-row">
      <input
        class="title-input"
        type="text"
        placeholder="標題"
        value={title}
        on:input={handleTitleInput}
      />
    </div>

    <Toolbar
      editor={editorRef?.getEditor()}
      onOpenSettings={() => settingsOpen = !settingsOpen}
    />

    <div class="editor-area">
      <Editor
        bind:this={editorRef}
        content={body}
        fontSize={note.settings.fontSize}
        on:update={handleContentUpdate}
      />
      {#if settingsOpen}
        <SettingsPanel
          settings={note.settings}
          {noteId}
          {contextId}
          on:change={handleSettingsChange}
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
    flex: 1;
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
  }
  .title-input {
    width: 100%;
    background: transparent;
    border: none;
    color: var(--text-primary);
    font-size: 16px;
    font-weight: 600;
    padding: 2px 0;
  }
  .title-input:focus { outline: none; border-bottom: 1px solid var(--accent); }
  .editor-area { display: flex; flex: 1; overflow: hidden; }
</style>
