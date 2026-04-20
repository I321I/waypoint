<script lang="ts">
  import { onMount } from "svelte";
  import { emit } from "@tauri-apps/api/event";
  import Editor from "./note/Editor.svelte";
  import Toolbar from "./note/Toolbar.svelte";
  import SettingsPanel from "./note/SettingsPanel.svelte";
  import { notes as notesApi, windows as windowsApi } from "../lib/api";
  import type { Note, NoteSettings } from "../lib/types";
  import { parseTitleContent, joinTitleContent } from "../lib/noteFormat";

  export let noteId: string;
  export let contextId: string | null;

  let note: Note | null = null;
  let title: string = "";
  let body: string = "";
  let settingsOpen = false;
  let editorRef: Editor;
  let saveTimeout: ReturnType<typeof setTimeout>;
  let windowOpacity = 1;
  void windowOpacity;

  function applyOpacity(opacity: number) {
    windowOpacity = opacity;
    document.documentElement.style.opacity = String(opacity);
  }

  onMount(async () => {
    note = await notesApi.read(contextId, noteId);
    if (note) {
      applyOpacity(note.settings.opacity);
      const parsed = parseTitleContent(note.content);
      title = parsed.title || note.title || "";
      body = parsed.body;
    }
  });

  function scheduleSave() {
    if (!note) return;
    clearTimeout(saveTimeout);
    saveTimeout = setTimeout(async () => {
      const merged = joinTitleContent(title, body);
      await notesApi.saveContent(contextId, noteId, merged);
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

  async function handleMinimize() {
    await windowsApi.minimizeWindow(`note-${noteId}`);
  }

  async function handleMaximize() {
    await windowsApi.toggleMaximize(`note-${noteId}`);
  }

  // Fallback：data-tauri-drag-region 有時被 child 吃掉 mousedown（buttons、span overflow
  // 等），導致拖曳失效。這裡在 titlebar 空白區 mousedown 時直接呼叫 start_dragging。
  function handleTitlebarMousedown(e: MouseEvent) {
    if (e.button !== 0) return;
    const target = e.target as HTMLElement;
    if (target.closest("button") || target.closest("input")) return;
    windowsApi.startDragging(`note-${noteId}`).catch(() => {});
  }
</script>

{#if note}
  <div class="note-window">
    <div class="titlebar" data-tauri-drag-region on:mousedown={handleTitlebarMousedown}>
      <span class="note-title" data-tauri-drag-region>{title || "Untitled"}{contextId ? ` — ${contextId}` : ""}</span>
      <div class="titlebar-buttons">
        <button on:click={handleMinimize} title="最小化">—</button>
        <button on:click={handleMaximize} title="最大化／還原">▢</button>
        <button on:click={handleClose} title="儲存並關閉">✕</button>
      </div>
    </div>

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
  .note-window {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-primary);
    border: 1px solid var(--border);
  }
  .titlebar {
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
  .titlebar:active { cursor: grabbing; }
  .note-title {
    font-size: 12px;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    pointer-events: none;  /* 讓 mousedown 打到 .titlebar 本體而非 span */
  }
  .titlebar-buttons { display: flex; gap: 6px; flex-shrink: 0; }
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
