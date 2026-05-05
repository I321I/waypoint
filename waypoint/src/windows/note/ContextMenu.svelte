<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from "svelte";
  import type { Editor } from "@tiptap/core";
  import ConfirmDialog from "../ConfirmDialog.svelte";
  import { notes as notesApi } from "../../lib/api";

  export let editor: Editor | null;
  export let fontSize: number;
  export let noteId: string;
  export let contextId: string | null;
  export let x: number;
  export let y: number;

  const dispatch = createEventDispatcher<{
    close: void;
    'font-size-change': number;
  }>();

  let confirmingDelete = false;

  function close() { dispatch('close'); }

  async function copyAction() {
    if (!editor) { close(); return; }
    const { from, to } = editor.state.selection;
    if (from === to) { close(); return; }
    const text = editor.state.doc.textBetween(from, to, '\n');
    try { await navigator.clipboard.writeText(text); } catch {}
    close();
  }

  async function pasteAction() {
    if (!editor) { close(); return; }
    try {
      const text = await navigator.clipboard.readText();
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (editor.commands as any).insertContent(text);
    } catch {}
    close();
  }

  function fontStep(delta: number) {
    const next = Math.max(8, Math.min(32, fontSize + delta));
    fontSize = next;
    dispatch('font-size-change', next);
  }

  async function doDelete() {
    confirmingDelete = false;
    await notesApi.delete(contextId, noteId);
    close();
  }

  function handleKey(e: KeyboardEvent) {
    if (e.key === 'Escape') close();
  }

  function handleClickOutside(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (!target.closest('.context-menu') && !target.closest('.dialog') && !target.closest('.overlay')) close();
  }

  onMount(() => {
    document.addEventListener('keydown', handleKey);
    document.addEventListener('mousedown', handleClickOutside);
  });
  onDestroy(() => {
    document.removeEventListener('keydown', handleKey);
    document.removeEventListener('mousedown', handleClickOutside);
  });
</script>

<div class="context-menu" style="left: {x}px; top: {y}px;">
  <button data-mi="copy" on:click={copyAction}>複製</button>
  <button data-mi="paste" on:click={pasteAction}>貼上</button>
  <div data-mi="font-size" class="stepper-row">
    <span>字體大小</span>
    <button data-act="dec" on:click={() => fontStep(-1)}>−</button>
    <span class="num">{fontSize}</span>
    <button data-act="inc" on:click={() => fontStep(1)}>+</button>
  </div>
  <button data-mi="delete" class="danger" on:click={() => confirmingDelete = true}>刪除此筆記</button>
</div>

{#if confirmingDelete}
  <ConfirmDialog
    message="確定要刪除這份筆記？此操作無法復原。"
    confirmText="刪除"
    cancelText="取消"
    onConfirm={doDelete}
    onCancel={() => confirmingDelete = false}
  />
{/if}

<style>
  .context-menu {
    position: fixed;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 4px;
    box-shadow: 0 2px 8px rgba(0,0,0,.4);
    padding: 4px 0;
    min-width: 160px;
    z-index: 1100;
    display: flex;
    flex-direction: column;
    font-size: 12px;
  }
  .context-menu button {
    text-align: left;
    background: transparent;
    border: none;
    color: var(--text-primary);
    padding: 6px 12px;
    cursor: pointer;
  }
  .context-menu button:hover { background: var(--bg-tertiary); }
  .context-menu button.danger { color: var(--accent-danger, #c0392b); }
  .stepper-row {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 12px;
    border-top: 1px solid var(--border);
    border-bottom: 1px solid var(--border);
  }
  .stepper-row > span:first-child { flex: 1; }
  .stepper-row .num { flex: 0 0 28px; text-align: center; }
  .stepper-row button { padding: 0 6px; font-size: 12px; }
</style>
