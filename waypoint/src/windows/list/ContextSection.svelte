<script lang="ts">
  import type { Note } from "../../lib/types";
  import NoteItem from "./NoteItem.svelte";
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

  async function showContextMenu(e: MouseEvent) {
    e.preventDefault();
    menuX = e.clientX;
    menuY = e.clientY;
    availableContexts = (await contextApi.listAll()).filter(c => c !== contextId);
    menuVisible = true;
  }

  function closeMenu() { menuVisible = false; showAliasInput = false; }

  async function setMatchBy(matchBy: "process" | "title") {
    await contextApi.setMatchBy(contextId, matchBy);
    closeMenu();
  }

  async function setAlias(target: string) {
    await contextApi.setAlias(contextId, target);
    dispatch("aliasSet", { from: contextId, to: target });
    closeMenu();
  }

  async function deleteCtx() {
    if (confirm(`刪除 context "${contextId}" 及其所有筆記？`)) {
      await contextApi.delete(contextId);
      dispatch("deleted", { contextId });
    }
    closeMenu();
  }

  async function addNote() {
    const note = await notesApi.create(contextId, "New Note");
    notes = [...notes, note];
  }

  function handleOpened(e: CustomEvent) {
    dispatch("opened", e.detail);
  }
</script>

<svelte:window on:click={closeMenu} />

<div class="section">
  <div class="section-header" on:contextmenu={showContextMenu}>
    <span class="section-label">{contextId} 筆記</span>
    <button class="add-btn" on:click|stopPropagation={addNote} title="新增筆記">+</button>
  </div>
  {#each notes as note (note.id)}
    <NoteItem {note} isOpen={openNoteIds.includes(note.id)} on:opened={handleOpened} />
  {/each}
</div>

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
