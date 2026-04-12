<script lang="ts">
  import type { Note } from "../../lib/types";
  import NoteItem from "./NoteItem.svelte";
  import { notes as notesApi } from "../../lib/api";
  import { createEventDispatcher } from "svelte";

  export let notes: Note[] = [];
  export let openNoteIds: string[] = [];

  const dispatch = createEventDispatcher();

  async function addNote() {
    const note = await notesApi.create(null, "New Note");
    notes = [...notes, note];
  }

  function handleOpened(e: CustomEvent) {
    dispatch("opened", e.detail);
  }
</script>

<div class="section">
  <div class="section-header">
    <span class="section-label">🌐 全域筆記</span>
    <button class="add-btn" on:click={addNote} title="新增全域筆記">+</button>
  </div>
  {#each notes as note (note.id)}
    <NoteItem {note} isOpen={openNoteIds.includes(note.id)} on:opened={handleOpened} />
  {/each}
</div>

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
  }
  .add-btn {
    font-size: 14px;
    color: var(--text-secondary);
    padding: 0 4px;
    line-height: 1;
  }
  .add-btn:hover { color: var(--text-primary); background: none; }
</style>
