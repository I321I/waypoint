<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import GlobalSection from "./list/GlobalSection.svelte";
  import ContextSection from "./list/ContextSection.svelte";
  import { notes as notesApi, context as contextApi, session as sessionApi, windows as windowsApi } from "../lib/api";
  import { globalNotes, contextNotes, activeContextId } from "../lib/stores";

  let currentContextId: string | null = null;
  let openGlobalNoteIds: string[] = [];
  let openContextNoteIds: string[] = [];
  let unlisten: (() => void) | null = null;
  let unlistenCollapse: (() => void) | null = null;

  function handleNoteOpened(noteId: string, isGlobal: boolean) {
    if (isGlobal) {
      if (!openGlobalNoteIds.includes(noteId))
        openGlobalNoteIds = [...openGlobalNoteIds, noteId];
    } else {
      if (!openContextNoteIds.includes(noteId))
        openContextNoteIds = [...openContextNoteIds, noteId];
    }
  }

  function handleNoteClosed(noteId: string, isGlobal: boolean) {
    if (isGlobal) {
      openGlobalNoteIds = openGlobalNoteIds.filter(id => id !== noteId);
    } else {
      openContextNoteIds = openContextNoteIds.filter(id => id !== noteId);
    }
  }

  onMount(async () => {
    currentContextId = await contextApi.getActive();
    activeContextId.set(currentContextId);

    const [globals, contexts] = await Promise.all([
      notesApi.list(null),
      currentContextId ? notesApi.list(currentContextId) : Promise.resolve([]),
    ]);
    globalNotes.set(globals);
    contextNotes.set(contexts);

    if (currentContextId) {
      const sess = await sessionApi.load(currentContextId);
      openContextNoteIds = sess.openContextNotes;
      openGlobalNoteIds = sess.openGlobalNotes;
      for (const nId of sess.openContextNotes) {
        await windowsApi.openNote(nId, currentContextId);
      }
      for (const nId of sess.openGlobalNotes) {
        await windowsApi.openNote(nId, null);
      }
    }

    // Listen for note-closed events from note windows
    unlisten = await listen<{ noteId: string; isGlobal: boolean }>("note-closed", (event) => {
      handleNoteClosed(event.payload.noteId, event.payload.isGlobal);
    });

    // Listen for collapse-all-requested event from hotkey handler so session is saved first
    unlistenCollapse = await listen("waypoint://collapse-all-requested", async () => {
      await handleCollapseAll();
    });
  });

  onDestroy(() => {
    unlisten?.();
    unlistenCollapse?.();
  });

  async function handleCollapseAll() {
    if (currentContextId) {
      await sessionApi.save(currentContextId, {
        openContextNotes: openContextNoteIds,
        openGlobalNotes: openGlobalNoteIds,
      });
    }
    await windowsApi.collapseAll();
  }

  async function openHelp() {
    const { invoke } = await import("@tauri-apps/api/core");
    invoke("cmd_open_help").catch(() => {});
  }

  function openSettings() {
    windowsApi.openSettings().catch(() => {});
  }

  function closeList() { windowsApi.hideWindow("list").catch(() => {}); }
</script>

<div class="list-window">
  <div class="titlebar" data-tauri-drag-region>
    <div class="titlebar-left">
      <span class="app-name">WAYPOINT</span>
      <button class="icon-btn" on:click={openHelp} title="使用說明">?</button>
    </div>
    <div class="titlebar-right">
      <button class="icon-btn" on:click={openSettings} title="設定">⚙</button>
      <button class="icon-btn" on:click={handleCollapseAll} title="收起全部">⇊</button>
      <button class="icon-btn" on:click={closeList} title="關閉列表">✕</button>
    </div>
  </div>

  <div class="list-body">
    <GlobalSection
      notes={$globalNotes}
      openNoteIds={openGlobalNoteIds}
      on:opened={(e) => handleNoteOpened(e.detail.noteId, e.detail.isGlobal)}
    />
    <div class="divider"></div>
    {#if currentContextId}
      <ContextSection
        contextId={currentContextId}
        notes={$contextNotes}
        openNoteIds={openContextNoteIds}
        on:opened={(e) => handleNoteOpened(e.detail.noteId, e.detail.isGlobal)}
        on:deleted={() => contextNotes.set([])}
      />
    {/if}
  </div>
</div>

<style>
  .list-window {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
  }
  .titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 5px 10px;
    background: var(--bg-tertiary);
    border-bottom: 1px solid var(--border);
    min-height: 32px;
  }
  .titlebar-left { display: flex; align-items: center; gap: 8px; }
  .titlebar-right { display: flex; align-items: center; gap: 6px; }
  .app-name { font-size: 11px; font-weight: bold; color: var(--text-primary); letter-spacing: 1px; }
  .icon-btn { font-size: 12px; padding: 2px 5px; }
  .list-body { flex: 1; overflow-y: auto; padding: 4px 0; }
</style>
