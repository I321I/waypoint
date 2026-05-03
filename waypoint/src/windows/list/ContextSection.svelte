<script lang="ts">
  import type { Note } from "../../lib/types";
  import NoteItem from "./NoteItem.svelte";
  import ConfirmDialog from "../ConfirmDialog.svelte";
  import { context as contextApi, notes as notesApi } from "../../lib/api";
  import { createEventDispatcher } from "svelte";

  export let contextId: string;
  export let notes: Note[] = [];
  export let openNoteIds: string[] = [];

  const dispatch = createEventDispatcher();

  let menuVisible = false;
  let menuX = 0;
  let menuY = 0;
  let showAliasInput = false;
  let aliasTarget = "";
  let availableContexts: string[] = [];
  let showRenameInput = false;
  let newContextName = "";
  let showDeleteConfirm = false;

  async function showContextMenu(e: MouseEvent) {
    e.preventDefault();
    menuX = e.clientX;
    menuY = e.clientY;
    availableContexts = (await contextApi.listAll()).filter(c => c !== contextId);
    menuVisible = true;
  }

  function closeMenu() { menuVisible = false; showAliasInput = false; showRenameInput = false; }

  async function renameCtx() {
    if (!newContextName.trim() || newContextName === contextId) {
      showRenameInput = false;
      return;
    }
    await contextApi.rename(contextId, newContextName.trim());
    dispatch("renamed", { oldId: contextId, newId: newContextName.trim() });
    closeMenu();
    showRenameInput = false;
  }

  async function setMatchBy(matchBy: "process" | "title") {
    await contextApi.setMatchBy(contextId, matchBy);
    closeMenu();
  }

  async function setAlias(target: string) {
    await contextApi.setAlias(contextId, target);
    dispatch("aliasSet", { from: contextId, to: target });
    closeMenu();
  }

  function deleteCtx() {
    showDeleteConfirm = true;
    closeMenu();
  }

  async function confirmDelete() {
    await contextApi.delete(contextId);
    dispatch("deleted", { contextId });
    showDeleteConfirm = false;
  }

  async function addNote() {
    const note = await notesApi.create(contextId, "New Note");
    notes = [...notes, note];
    await notesApi.setOrder(contextId, notes.map((n) => n.id));
  }

  function handleOpened(e: CustomEvent) {
    dispatch("opened", e.detail);
  }

  let draggingId: string | null = null;
  function onDragstart(e: CustomEvent<{ noteId: string }>) { draggingId = e.detail.noteId; }
  async function onDrop(e: CustomEvent<{ noteId: string }>) {
    const targetId = e.detail.noteId;
    if (!draggingId || draggingId === targetId) { draggingId = null; return; }
    const srcIdx = notes.findIndex((n) => n.id === draggingId);
    const dstIdx = notes.findIndex((n) => n.id === targetId);
    if (srcIdx < 0 || dstIdx < 0) { draggingId = null; return; }
    const next = notes.slice();
    const [moved] = next.splice(srcIdx, 1);
    next.splice(dstIdx, 0, moved);
    notes = next;
    draggingId = null;
    await notesApi.setOrder(contextId, next.map((n) => n.id));
  }
</script>

<svelte:window on:click={closeMenu} />

<div class="section">
  <div class="section-header" on:contextmenu={showContextMenu}>
    <span class="section-label">{contextId} 筆記</span>
    <button class="add-btn" on:click|stopPropagation={addNote} title="新增筆記">+</button>
  </div>
  {#each notes as note (note.id)}
    <NoteItem
      {note}
      isOpen={openNoteIds.includes(note.id)}
      on:opened={handleOpened}
      on:changed={() => dispatch("changed")}
      on:dragstart={onDragstart}
      on:drop={onDrop}
    />
  {/each}
</div>

{#if showDeleteConfirm}
  <ConfirmDialog
    message={`刪除 context "${contextId}" 及其所有筆記？`}
    confirmText="刪除"
    cancelText="取消"
    onConfirm={confirmDelete}
    onCancel={() => { showDeleteConfirm = false; }}
  />
{/if}

{#if menuVisible}
  <div
    class="context-menu"
    style="left:{menuX}px;top:{menuY}px"
    on:click|stopPropagation={() => {}}
  >
    <button on:click={() => setMatchBy("process")}>識別方式：程序名稱</button>
    <button on:click={() => setMatchBy("title")}>識別方式：視窗標題</button>
    <div class="divider" />
    {#if !showAliasInput}
      <button on:click={() => showAliasInput = true}>對應到現有 context...</button>
    {:else}
      <select class="alias-select" bind:value={aliasTarget}>
        <option value="">選擇 context</option>
        {#each availableContexts as ctx}
          <option value={ctx}>{ctx}</option>
        {/each}
      </select>
      <button on:click={() => aliasTarget && setAlias(aliasTarget)}>確認</button>
    {/if}
    <div class="divider" />
    {#if !showRenameInput}
      <button on:click={() => { showRenameInput = true; newContextName = contextId; }}>重新命名</button>
    {:else}
      <input
        class="alias-select"
        bind:value={newContextName}
        placeholder="新名稱"
        on:keydown={e => e.key === 'Enter' && renameCtx()}
      />
      <button on:click={renameCtx}>確認</button>
    {/if}
    <div class="divider" />
    <button class="danger" on:click={deleteCtx}>刪除此 context</button>
  </div>
{/if}

<style>
  .section { padding: 4px 0; }
  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 3px 10px;
    color: var(--text-secondary);
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 1px;
    margin-top: 4px;
    cursor: context-menu;
  }
  .section-header:hover { background: var(--bg-hover); }
  .add-btn { font-size: 14px; color: var(--text-secondary); padding: 0 4px; }
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
  }
  .context-menu button:hover { background: var(--bg-selected); }
  .context-menu .danger { color: var(--danger); }
  .alias-select {
    display: block;
    width: calc(100% - 16px);
    margin: 4px 8px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    color: var(--text-primary);
    padding: 3px 6px;
    font-size: 12px;
  }
</style>
