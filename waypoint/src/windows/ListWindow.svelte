<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import GlobalSection from "./list/GlobalSection.svelte";
  import ContextSection from "./list/ContextSection.svelte";
  import DraggableTitlebar from "./DraggableTitlebar.svelte";
  import { notes as notesApi, context as contextApi, session as sessionApi, windows as windowsApi } from "../lib/api";
  import { globalNotes, contextNotes, activeContextId } from "../lib/stores";

  let currentContextId: string | null = null;
  let openGlobalNoteIds: string[] = [];
  let openContextNoteIds: string[] = [];
  let unlisten: (() => void) | null = null;
  let unlistenCollapse: (() => void) | null = null;
  let unlistenShown: (() => void) | null = null;

  async function loadContextAndSession() {
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
    } else {
      openContextNoteIds = [];
      openGlobalNoteIds = [];
    }
  }

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
    await loadContextAndSession();

    unlisten = await listen<{ noteId: string; isGlobal: boolean }>("note-closed", (event) => {
      handleNoteClosed(event.payload.noteId, event.payload.isGlobal);
    });

    unlistenCollapse = await listen("waypoint://collapse-all-requested", async () => {
      await handleCollapseAll();
    });

    // 當 list 再次被叫出（Rust 端重用既存視窗），重新依當前前景 app 載入 context / session
    unlistenShown = await listen("waypoint://list-shown", async () => {
      await loadContextAndSession();
    });
  });

  onDestroy(() => {
    unlisten?.();
    unlistenCollapse?.();
    unlistenShown?.();
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

  function minimizeList() { windowsApi.minimizeWindow("list").catch(() => {}); }
  function quitApp() { windowsApi.exitApp().catch(() => {}); }

  async function reloadLists() {
    const [globals, contexts] = await Promise.all([
      notesApi.list(null),
      currentContextId ? notesApi.list(currentContextId) : Promise.resolve([]),
    ]);
    globalNotes.set(globals);
    contextNotes.set(contexts);
  }

</script>

<div class="list-window">
  <DraggableTitlebar label="list">
    <div class="titlebar-left" data-tauri-drag-region>
      <span class="app-name" data-tauri-drag-region>WAYPOINT</span>
      <button class="icon-btn" on:click={openHelp} title="使用說明">?</button>
    </div>
    <div class="titlebar-right">
      <button class="icon-btn" on:click={openSettings} title="設定">⚙</button>
      <button class="icon-btn" on:click={handleCollapseAll} title="收起全部">⇊</button>
      <button class="icon-btn" on:click={minimizeList} title="最小化列表">—</button>
      <button class="icon-btn" on:click={quitApp} title="結束 Waypoint">✕</button>
    </div>
  </DraggableTitlebar>

  <div class="list-body">
    <GlobalSection
      notes={$globalNotes}
      openNoteIds={openGlobalNoteIds}
      on:opened={(e) => handleNoteOpened(e.detail.noteId, e.detail.isGlobal)}
      on:changed={reloadLists}
    />
    <div class="divider"></div>
    {#if currentContextId}
      <ContextSection
        contextId={currentContextId}
        notes={$contextNotes}
        openNoteIds={openContextNoteIds}
        on:opened={(e) => handleNoteOpened(e.detail.noteId, e.detail.isGlobal)}
        on:deleted={() => contextNotes.set([])}
        on:changed={reloadLists}
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
  .titlebar-left { display: flex; align-items: center; gap: 8px; }
  .titlebar-right { display: flex; align-items: center; gap: 6px; }
  .app-name { font-size: 11px; font-weight: bold; color: var(--text-primary); letter-spacing: 1px; pointer-events: none; }
  .icon-btn { font-size: 12px; padding: 2px 5px; }
  .list-body { flex: 1; overflow-y: auto; padding: 4px 0; }
</style>
