<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { NoteSettings } from "../../lib/types";

  export let settings: NoteSettings;
  // R4：noteId/contextId 由 NoteWindow 傳入但目前面板不使用，保留 prop 以維持外部 API。
  export let noteId: string;
  export let contextId: string | null;
  void noteId; void contextId;
  const dispatch = createEventDispatcher<{ change: NoteSettings }>();

  function update(patch: Partial<NoteSettings>) {
    settings = { ...settings, ...patch };
    dispatch("change", settings);
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

</div>

<style>
  .settings-panel {
    width: 200px;
    background: var(--bg-secondary);
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
</style>
