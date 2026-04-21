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

  <div class="setting-row">
    <label>透明度</label>
    <div class="slider-row">
      <input
        class="opacity-slider"
        type="range" min="10" max="100" step="5"
        value={Math.round(settings.opacity * 100)}
        on:input={e => update({ opacity: parseInt((e.target as HTMLInputElement).value, 10) / 100 })}
      />
      <span class="value">{Math.round(settings.opacity * 100)}%</span>
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
  .slider-row { display: flex; align-items: center; gap: 8px; }
  .slider-row input { flex: 1; accent-color: var(--accent); }
  /* R6: 自訂 thumb 讓 100% 時對齊右邊界（原生 thumb 在 max 時會超出 track 一半） */
  .opacity-slider {
    appearance: none;
    -webkit-appearance: none;
    width: 100%;
    height: 4px;
    margin: 0;
    background: var(--border);
    border-radius: 2px;
    outline: none;
  }
  .opacity-slider::-webkit-slider-thumb {
    appearance: none;
    -webkit-appearance: none;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--accent);
    border: none;
    margin-top: 0;
    cursor: pointer;
  }
  .opacity-slider::-moz-range-thumb {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--accent);
    border: none;
    cursor: pointer;
  }
  .value { font-size: 11px; color: var(--text-secondary); min-width: 30px; }
</style>
