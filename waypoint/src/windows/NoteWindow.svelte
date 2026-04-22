<script lang="ts">
  import { onMount } from "svelte";
  import { emit } from "@tauri-apps/api/event";
  import Editor from "./note/Editor.svelte";
  import Toolbar from "./note/Toolbar.svelte";
  import SettingsPanel from "./note/SettingsPanel.svelte";
  import { notes as notesApi, windows as windowsApi } from "../lib/api";
  import type { Note, NoteSettings } from "../lib/types";

  export let noteId: string;
  export let contextId: string | null;

  let note: Note | null = null;
  let settingsOpen = false;
  let editorRef: Editor;
  let saveTimeout: ReturnType<typeof setTimeout>;
  let windowOpacity = 1;

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
    }
  });

  function handleContentUpdate(e: CustomEvent<{ markdown: string }>) {
    if (!note) return;
    clearTimeout(saveTimeout);
    saveTimeout = setTimeout(async () => {
      await notesApi.saveContent(contextId, noteId, e.detail.markdown);
    }, 500);
  }

  async function handleSettingsChange(e: CustomEvent<NoteSettings>) {
    if (!note) return;
    note = { ...note, settings: e.detail };
    await notesApi.saveSettings(contextId, noteId, e.detail);
    applyOpacity(e.detail.opacity);
  }

  async function flushPendingSave() {
    if (!editorRef) return;
    clearTimeout(saveTimeout);
    const ed = editorRef.getEditor();
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const md = (ed?.storage as any)?.markdown?.getMarkdown?.() ?? null;
    if (md !== null) {
      await notesApi.saveContent(contextId, noteId, md);
    }
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
    <div class="titlebar" data-tauri-drag-region>
      <span class="note-title" data-tauri-drag-region>{note.title || "Untitled"}{contextId ? ` — ${contextId}` : ""}</span>
      <div class="titlebar-buttons">
        <button on:click={handleCollapseAll} title="收起全部並儲存">⇊</button>
        <button on:click={handleMaximize} title="最大化／還原">▢</button>
        <button on:click={handleClose} title="儲存並關閉">✕</button>
      </div>
    </div>

    <Toolbar
      editor={editorRef?.getEditor()}
      onOpenSettings={() => settingsOpen = !settingsOpen}
    />

    <div class="editor-area">
      <Editor
        bind:this={editorRef}
        content={note.content}
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
  }
  .note-title {
    font-size: 12px;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }
  .titlebar-buttons { display: flex; gap: 6px; flex-shrink: 0; }
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
