<script lang="ts">
  export let message: string;
  export let confirmText: string = "確認";
  export let cancelText: string = "取消";
  export let onConfirm: () => void;
  export let onCancel: () => void;
</script>

<div class="overlay" on:click={onCancel}>
  <div class="dialog" on:click|stopPropagation={() => {}}>
    <p class="msg">{message}</p>
    <button class="danger" on:click={onConfirm}>{confirmText}</button>
    <button class="cancel" on:click={onCancel}>{cancelText}</button>
  </div>
</div>

<style>
  .overlay {
    position: fixed; inset: 0;
    background: rgba(0,0,0,.6);
    display: flex; align-items: center; justify-content: center;
    z-index: 1000;
  }
  .dialog {
    background: var(--bg-primary);
    padding: 12px;
    border-radius: 6px;
    width: calc(100% - 24px);
    max-width: 280px;
    display: flex; flex-direction: column; gap: 8px;
  }
  .msg { font-size: 12px; color: var(--text-primary); margin: 0; }
  .dialog button { width: 100%; padding: 6px; font-size: 12px; }
  .danger { background: var(--accent-danger, #c0392b); color: white; border: none; border-radius: 4px; }
  .cancel { background: var(--bg-tertiary); color: var(--text-primary); border: 1px solid var(--border); border-radius: 4px; }
</style>
