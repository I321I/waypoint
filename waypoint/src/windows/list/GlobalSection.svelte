<script lang="ts">
  import type { Note } from "../../lib/types";
  import NoteItem from "./NoteItem.svelte";
  import { notes as notesApi } from "../../lib/api";
  import { createEventDispatcher } from "svelte";

  export let notes: Note[] = [];
  export let openNoteIds: string[] = [];

  const dispatch = createEventDispatcher<{
    opened: { noteId: string; isGlobal: boolean };
    changed: void;
  }>();

  let draggingId: string | null = null;

  async function addNote() {
    const note = await notesApi.create(null, "New Note");
    notes = [...notes, note];
    // 附加到 order 尾端
    const order = notes.map((n) => n.id);
    await notesApi.setOrder(null, order);
  }

  function handleOpened(e: CustomEvent) {
    dispatch("opened", e.detail);
  }

  function onDragstart(e: CustomEvent<{ noteId: string }>) {
    draggingId = e.detail.noteId;
  }

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
    await notesApi.setOrder(null, next.map((n) => n.id));
  }
</script>

<div class="section">
  <div class="section-header">
    <span class="section-label">🌐 全域筆記</span>
    <button class="add-btn" on:click={addNote} title="新增全域筆記">+</button>
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
