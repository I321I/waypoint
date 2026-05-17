<script lang="ts">
  import { createEventDispatcher } from "svelte";
  export let opacity: number;
  const dispatch = createEventDispatcher<{ change: number }>();

  function onInput(e: Event) {
    const v = parseInt((e.target as HTMLInputElement).value, 10) / 100;
    dispatch("change", v);
  }
</script>

<div class="opacity-bar">
  <span class="lbl">透明度</span>
  <input
    class="slider"
    type="range" min="10" max="100" step="5"
    value={Math.round(opacity * 100)}
    on:input={onInput}
    title="視窗透明度 {Math.round(opacity * 100)}%"
  />
</div>

<style>
  .opacity-bar {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 1 1 auto;
    min-width: 0;
  }
  .lbl { font-size: 11px; color: var(--text-secondary); letter-spacing: 0.5px; flex-shrink: 0; }
  .slider {
    flex: 1 1 auto;
    min-width: 60px;
    appearance: none;
    -webkit-appearance: none;
    height: 4px;
    margin: 0;
    background: var(--border);
    border-radius: 2px;
    outline: none;
    accent-color: var(--accent);
  }
  .slider::-webkit-slider-thumb {
    appearance: none;
    -webkit-appearance: none;
    width: 12px; height: 12px;
    border-radius: 50%;
    background: var(--accent);
    border: none;
    cursor: pointer;
  }
  .slider::-moz-range-thumb {
    width: 12px; height: 12px;
    border-radius: 50%;
    background: var(--accent);
    border: none;
    cursor: pointer;
  }
</style>
