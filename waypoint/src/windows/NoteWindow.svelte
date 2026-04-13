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

  // 套用視窗透明度（CSS opacity，因 Tauri 2.x JS SDK 無 setOpacity）
  function applyOpacity(opacity: number) {
    windowOpacity = opacity;
    document.documentElement.style.opacity = String(opacity);
  }

  onMount(async () => {
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

  async function handleClose() {
    await emit("note-closed", { noteId, contextId, isGlobal: contextId === null });
    await windowsApi.closeNote(noteId);
  }

  async function handleMinimize() {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().minimize();
  }
</script>

{#if note}
  <div class="note-window">
    <div class="titlebar" data-tauri-drag-region>
      <span class="note-title">{note.title || "Untitled"}{contextId ? ` — ${contextId}` : ""}</span>
      <div class="titlebar-buttons">
        <button on:click={handleMinimize} title="最小化">—</button>
        <button on:click={handleClose} title="關閉">✕</button>
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
