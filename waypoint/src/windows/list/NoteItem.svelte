<script lang="ts">
  import type { Note } from "../../lib/types";
  import { windows, notes as notesApi, context as contextApi } from "../../lib/api";
  import { emit } from "@tauri-apps/api/event";
  import { createEventDispatcher } from "svelte";

  export let note: Note;
  // 保留 prop 相容性，不產生反藍
  export let isOpen: boolean = false;
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  void isOpen;

  const dispatch = createEventDispatcher<{
    opened: { noteId: string; isGlobal: boolean };
    changed: void;
    dragstart: { noteId: string };
    dragover: { noteId: string };
    drop: { noteId: string };
  }>();

  let menuVisible = false;
  let menuX = 0;
  let menuY = 0;
  let submenu: "copy" | "move" | null = null;
  let renaming = false;
  let renameValue = "";
  let availableContexts: string[] = [];

  async function handleClick() {
    if (renaming) return;
    await windows.openNote(note.id, note.contextId);
    dispatch("opened", { noteId: note.id, isGlobal: note.contextId === null });
  }

  async function openMenu(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    menuX = e.clientX;
    menuY = e.clientY;
    menuVisible = true;
    submenu = null;
    availableContexts = await contextApi.listAll();
  }

  function closeMenu() {
    menuVisible = false;
    submenu = null;
    renaming = false;
  }

  function startRename() {
    renaming = true;
    renameValue = note.title || "";
    menuVisible = false;
  }

  async function submitRename() {
    const newTitle = renameValue.trim();
    if (newTitle && newTitle !== note.title) {
      await notesApi.rename(note.contextId, note.id, newTitle);
      // 廣播給可能開著的 NoteWindow（不會回送 title-changed，避免 echo loop）
      await emit("waypoint://note-renamed-from-list", {
        noteId: note.id,
        contextId: note.contextId,
        title: newTitle,
      });
      dispatch("changed");
    }
    renaming = false;
  }

  async function doDuplicate(dstCtx: string | null) {
    await notesApi.duplicate(note.contextId, note.id, dstCtx);
    dispatch("changed");
    closeMenu();
  }

  async function doMove(dstCtx: string | null) {
    if (dstCtx === note.contextId) { closeMenu(); return; }
    await notesApi.move(note.contextId, note.id, dstCtx);
    dispatch("changed");
    closeMenu();
  }

  async function doDelete() {
    if (confirm(`刪除筆記「${note.title || "Untitled"}」？`)) {
      await notesApi.delete(note.contextId, note.id);
      dispatch("changed");
    }
    closeMenu();
  }

  function handleDragStart(e: DragEvent) {
    if (!e.dataTransfer) return;
    e.dataTransfer.effectAllowed = "move";
    e.dataTransfer.setData("text/x-waypoint-note", note.id);
    dispatch("dragstart", { noteId: note.id });
  }

  function handleDragOver(e: DragEvent) {
    if (!e.dataTransfer) return;
    const t = e.dataTransfer.types || [];
    if (!Array.from(t).includes("text/x-waypoint-note")) return;
    e.preventDefault();
    e.dataTransfer.dropEffect = "move";
    dispatch("dragover", { noteId: note.id });
  }

  function handleDrop(e: DragEvent) {
    if (!e.dataTransfer) return;
    const srcId = e.dataTransfer.getData("text/x-waypoint-note");
    if (!srcId) return;
    e.preventDefault();
    dispatch("drop", { noteId: note.id });
  }
</script>

<svelte:window on:click={closeMenu} />

{#if renaming}
  <div class="note-item renaming">
    <span class="icon">📄</span>
    <input
      class="rename-input"
      bind:value={renameValue}
      on:keydown={(e) => {
        if (e.key === "Enter") submitRename();
        else if (e.key === "Escape") renaming = false;
      }}
      on:blur={submitRename}
      autofocus
    />
  </div>
{:else}
  <div
    role="button"
    tabindex="0"
    class="note-item"
    draggable="true"
    data-note-id={note.id}
    on:click={handleClick}
    on:keydown={(e) => { if (e.key === "Enter") handleClick(); }}
    on:contextmenu={openMenu}
    on:dragstart={handleDragStart}
    on:dragover={handleDragOver}
    on:drop={handleDrop}
    title={note.title}
  >
    <span class="icon">📄</span>
    <span class="title">{note.title || "Untitled"}</span>
  </div>
{/if}

{#if menuVisible}
  <div class="context-menu" style="left:{menuX}px;top:{menuY}px" on:click|stopPropagation={() => {}}>
    <button on:click={startRename}>重新命名…</button>
    <div class="divider" />
    {#if submenu !== "copy"}
      <button on:click={() => submenu = "copy"}>複製到 ▸</button>
    {:else}
      <div class="submenu-label">複製到：</div>
      <button on:click={() => doDuplicate(null)}>🌐 全域</button>
      {#each availableContexts as ctx}
        <button on:click={() => doDuplicate(ctx)}>{ctx}</button>
      {/each}
    {/if}
    {#if submenu !== "move"}
      <button on:click={() => submenu = "move"}>移動到 ▸</button>
    {:else}
      <div class="submenu-label">移動到：</div>
      <button on:click={() => doMove(null)} disabled={note.contextId === null}>🌐 全域</button>
      {#each availableContexts as ctx}
        <button on:click={() => doMove(ctx)} disabled={note.contextId === ctx}>{ctx}</button>
      {/each}
    {/if}
    <div class="divider" />
    <button class="danger" on:click={doDelete}>刪除筆記</button>
  </div>
{/if}

<style>
  .note-item {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 5px 10px;
    text-align: left;
    color: var(--text-primary);
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    cursor: pointer;
  }
  .note-item:hover { background: var(--bg-hover); }
  .note-item.renaming { background: var(--bg-hover); cursor: text; }
  .icon { color: var(--text-link); font-size: 11px; flex-shrink: 0; }
  .title { overflow: hidden; text-overflow: ellipsis; }
  .rename-input {
    flex: 1;
    background: var(--bg-primary);
    border: 1px solid var(--accent);
    color: var(--text-primary);
    font-size: 12px;
    padding: 2px 4px;
  }
  .context-menu {
    position: fixed;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    z-index: 9999;
    min-width: 180px;
    padding: 4px 0;
    box-shadow: 0 4px 12px rgba(0,0,0,0.4);
  }
  .context-menu button {
    display: block;
    width: 100%;
    text-align: left;
    padding: 5px 12px;
    border-radius: 0;
    color: var(--text-primary);
    font-size: 12px;
    background: none;
    border: none;
    cursor: pointer;
  }
  .context-menu button:hover:not(:disabled) { background: var(--bg-selected); }
  .context-menu button:disabled { color: var(--text-secondary); cursor: default; }
  .context-menu .danger { color: var(--danger); }
  .context-menu .submenu-label {
    padding: 4px 12px;
    color: var(--text-secondary);
    font-size: 10px;
    text-transform: uppercase;
  }
  .context-menu .divider {
    height: 1px;
    background: var(--border);
    margin: 4px 0;
  }
</style>
