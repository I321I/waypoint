<script lang="ts">
  import type { Note } from "../../lib/types";
  import { windows } from "../../lib/api";
  import { createEventDispatcher } from "svelte";

  export let note: Note;
  // 保留 prop 相容性但不再用來產生反藍：使用者希望點開筆記後列表項目回到正常狀態
  export let isOpen: boolean = false;
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  void isOpen;

  const dispatch = createEventDispatcher();

  async function handleClick() {
    await windows.openNote(note.id, note.contextId);
    dispatch("opened", { noteId: note.id, isGlobal: note.contextId === null });
  }
</script>

<button
  class="note-item"
  on:click={handleClick}
  title={note.title}
>
  <span class="icon">📄</span>
  <span class="title">{note.title || "Untitled"}</span>
</button>

<style>
  .note-item {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 5px 10px;
    text-align: left;
    color: var(--text-primary);
    border-radius: 0;
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .note-item:hover { background: var(--bg-hover); }
  .note-item.open { background: var(--bg-selected); }
  .icon { color: var(--text-link); font-size: 11px; flex-shrink: 0; }
  .title { overflow: hidden; text-overflow: ellipsis; }
</style>
