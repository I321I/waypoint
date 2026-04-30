<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { emit, listen } from "@tauri-apps/api/event";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import Editor from "./note/Editor.svelte";
  import Toolbar from "./note/Toolbar.svelte";
  import SettingsPanel from "./note/SettingsPanel.svelte";
  import TitlebarOpacitySlider from "./note/TitlebarOpacitySlider.svelte";
  import DraggableTitlebar from "./DraggableTitlebar.svelte";
  import { notes as notesApi, passthrough as passthroughApi, windows as windowsApi } from "../lib/api";
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
  let windowOpacity = 1;
  let passthrough = false;
  let unlistenPassthrough: UnlistenFn | null = null;
  let unlistenRenamedFromList: UnlistenFn | null = null;
  void windowOpacity;

  // 套用視窗透明度：改用 CSS variable 控制 body rgba 背景，避免文字也被淡化
  function applyOpacity(opacity: number) {
    windowOpacity = opacity;
    document.documentElement.style.setProperty('--note-alpha', String(opacity));
  }

  onMount(async () => {
    // 同步加上 class（不等 await），避免閃爍
    document.body.classList.add('note-view');
    note = await notesApi.read(contextId, noteId);
    if (note) {
      applyOpacity(note.settings.opacity);
      passthrough = note.settings.passthrough ?? false;
      const parsed = parseTitleContent(note.content);
      title = parsed.title || note.title || "";
      body = parsed.body;
      lastEmittedTitle = title;
    }
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
  });

  onDestroy(() => {
    unlistenPassthrough?.();
    unlistenRenamedFromList?.();
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
    }, 500);
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
    applyOpacity(e.detail.opacity);
  }

  async function flushPendingSave() {
    clearTimeout(saveTimeout);
    if (editorRef) {
      const ed = editorRef.getEditor();
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const md = (ed?.storage as any)?.markdown?.getMarkdown?.() ?? null;
      if (md !== null) body = md;
    }
    const merged = joinTitleContent(title, body);
    await notesApi.saveContent(contextId, noteId, merged);
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
  <div class="note-window">
    <DraggableTitlebar label={`note-${noteId}`}>
      <span class="note-title" data-tauri-drag-region>{title || "Untitled"}{contextId ? ` — ${contextId}` : ""}</span>
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

    <TitlebarOpacitySlider
      opacity={note.settings.opacity}
      on:change={async (e) => {
        if (!note) return;
        const next = { ...note.settings, opacity: e.detail };
        note = { ...note, settings: next };
        applyOpacity(e.detail);
        await notesApi.saveSettings(contextId, noteId, next);
      }}
    />

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

    <div class="statusbar">
      <span>{contextId ?? "Global"}</span>
      <span>Markdown</span>
    </div>
  </div>
{/if}

<style>
  :global(body.note-view) {
    background: rgba(30, 30, 30, var(--note-alpha, 1)) !important;
  }

  .note-window {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: rgba(30, 30, 30, var(--note-alpha, 1));
    border: 1px solid var(--border);
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
    background: var(--bg-primary);
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
  .statusbar {
    display: flex;
    justify-content: space-between;
    padding: 2px 10px;
    background: var(--accent);
    color: white;
    font-size: 11px;
  }
</style>
