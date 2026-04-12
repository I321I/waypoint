<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { NoteSettings } from "../../lib/types";
  import { windows as windowsApi } from "../../lib/api";

  export let settings: NoteSettings;
  export let noteId: string;
  export let contextId: string | null;
  const dispatch = createEventDispatcher<{ change: NoteSettings }>();

  function update(patch: Partial<NoteSettings>) {
    settings = { ...settings, ...patch };
    dispatch("change", settings);
  }

  async function handleHotkeyChange(e: Event) {
    const hotkey = (e.target as HTMLInputElement).value.trim() || null;
    if (settings.hotkey) {
      await windowsApi.unregisterHotkey(settings.hotkey).catch(() => {});
    }
    update({ hotkey });
    if (hotkey) {
      await windowsApi.registerNoteHotkey(noteId, contextId, hotkey);
    }
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
        type="range" min="0.1" max="1" step="0.05"
        value={settings.opacity}
        on:input={e => update({ opacity: parseFloat((e.target as HTMLInputElement).value) })}
      />
      <span class="value">{Math.round(settings.opacity * 100)}%</span>
    </div>
  </div>

  <div class="setting-row">
    <label>專屬快捷鍵</label>
    <input
      type="text"
      placeholder="留空不設定"
      value={settings.hotkey ?? ""}
      on:change={handleHotkeyChange}
    />
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
  .value { font-size: 11px; color: var(--text-secondary); min-width: 30px; }
</style>
