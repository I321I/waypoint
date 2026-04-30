<script lang="ts">
  import { windows as windowsApi } from "../lib/api";
  export let label: string;

  // 同步 fire-and-forget — async/await 在 mousedown handler 內會引入微 task 延遲，
  // 導致 start_dragging 在 mouse 事件迴圈結束後才執行；NoteWindow 已驗證此 pattern 可動。
  function handleMousedown(e: MouseEvent) {
    if (e.button !== 0) return;
    const target = e.target as HTMLElement;
    if (target.closest("button, input, textarea, select, a")) return;
    windowsApi.startDragging(label).catch(() => {});
  }
</script>

<div class="draggable-titlebar" data-tauri-drag-region on:mousedown={handleMousedown}>
  <slot />
</div>

<style>
  .draggable-titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 5px 10px;
    background: var(--bg-tertiary);
    border-bottom: 1px solid var(--border);
    min-height: 30px;
    gap: 8px;
    cursor: grab;
  }
  .draggable-titlebar:active { cursor: grabbing; }
</style>
