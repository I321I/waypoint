<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { NoteSettings } from "../../lib/types";
  import { notes as notesApi } from "../../lib/api";
  import ConfirmDialog from "../ConfirmDialog.svelte";

  export let settings: NoteSettings;
  export let noteId: string;
  export let contextId: string | null;
  const dispatch = createEventDispatcher<{ change: NoteSettings }>();

  let confirmingDelete = false;

  function update(patch: Partial<NoteSettings>) {
    settings = { ...settings, ...patch };
    dispatch("change", settings);
  }

  async function doDelete() {
    confirmingDelete = false;
    await notesApi.delete(contextId, noteId);
    // backend emits waypoint://note-deleted → NoteWindow self-closes
  }
</script>

<div class="settings-panel">
  <div class="setting-row">
    <label>字體大小</label>
    <div class="number-input">
      <button on:click={() => update({ fontSize: Math.max(8, settings.fontSize - 1) })}>-</button>
      <input
        type="number"
        min="8" max="32"
        value={settings.fontSize}
        on:change={e => update({ fontSize: parseInt((e.target as HTMLInputElement).value) })}
      />
      <button on:click={() => update({ fontSize: Math.min(32, settings.fontSize + 1) })}>+</button>
    </div>
  </div>

  <div class="danger-zone">
    <button
      class="danger-btn"
      data-testid="delete-this-note"
      on:click={() => confirmingDelete = true}
    >
      刪除此筆記
    </button>
  </div>
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
  .settings-panel {
    width: 200px;
    background: transparent;
    border-left: 1px solid var(--border);
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    overflow-y: auto;
  }
  .setting-row { display: flex; flex-direction: column; gap: 6px; }
  label { font-size: 11px; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.5px; }
  .number-input { display: flex; align-items: center; gap: 4px; }
  .number-input input { width: 48px; text-align: center; }
  .number-input button { padding: 2px 7px; }

  .danger-zone {
    margin-top: auto;
    padding-top: 12px;
    border-top: 1px solid var(--border);
  }
  .danger-btn {
    width: 100%;
    padding: 8px;
    background: var(--accent-danger, #c0392b);
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }
  .danger-btn:hover {
    background: #a83020;
  }
</style>
