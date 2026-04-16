<script lang="ts">
  import { onMount } from "svelte";
  import { config as configApi, windows as windowsApi } from "../lib/api";

  let hotkey = "";
  let hotkeyInput = "";
  let autostartEnabled = false;
  let autostartSupported = false;
  let saving = false;
  let message = "";

  onMount(async () => {
    const [cfg, supported, autostart] = await Promise.all([
      configApi.get(),
      configApi.isAutostartSupported(),
      configApi.getAutostart(),
    ]);
    hotkey = cfg.hotkey;
    hotkeyInput = cfg.hotkey;
    autostartSupported = supported;
    autostartEnabled = autostart;
  });

  async function saveHotkey() {
    if (!hotkeyInput.trim()) return;
    saving = true;
    try {
      await configApi.setHotkey(hotkeyInput.trim());
      hotkey = hotkeyInput.trim();
      message = "快捷鍵已儲存，重新啟動後生效";
    } catch (e) {
      message = `儲存失敗：${e}`;
    } finally {
      saving = false;
    }
  }

  async function toggleAutostart() {
    try {
      await configApi.setAutostart(!autostartEnabled);
      autostartEnabled = !autostartEnabled;
    } catch (e) {
      message = `設定失敗：${e}`;
    }
  }
</script>

<div class="settings-window">
  <div class="titlebar" data-tauri-drag-region>
    <span class="title">Waypoint — 設定</span>
    <button class="close-btn" on:click={() => windowsApi.closeWindow("settings").catch(() => {})}>✕</button>
  </div>

  <div class="content">
    <section>
      <h2>全域快捷鍵</h2>
      <p class="desc">按下快捷鍵開啟／收起 Waypoint 列表</p>
      <div class="row">
        <input
          type="text"
          bind:value={hotkeyInput}
          placeholder="例如：Ctrl+Shift+Space"
          class="hotkey-input"
        />
        <button on:click={saveHotkey} disabled={saving || hotkeyInput === hotkey}>
          {saving ? "儲存中…" : "套用"}
        </button>
      </div>
      <p class="hint">格式：Ctrl+Shift+Space、Alt+F1 等</p>
    </section>

    {#if autostartSupported}
      <section>
        <h2>開機自動啟動</h2>
        <p class="desc">系統開機後自動在背景啟動 Waypoint</p>
        <div class="row">
          <label class="toggle">
            <input
              type="checkbox"
              checked={autostartEnabled}
              on:change={toggleAutostart}
            />
            <span class="toggle-label">
              {autostartEnabled ? "已啟用" : "已停用"}
            </span>
          </label>
        </div>
      </section>
    {/if}

    {#if message}
      <p class="message">{message}</p>
    {/if}
  </div>
</div>

<style>
  .settings-window {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-primary);
  }
  .titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    background: var(--bg-tertiary);
    border-bottom: 1px solid var(--border);
  }
  .title { font-size: 12px; font-weight: bold; color: var(--text-primary); }
  .close-btn { font-size: 12px; padding: 2px 6px; }
  .content {
    flex: 1;
    overflow-y: auto;
    padding: 20px 24px;
    display: flex;
    flex-direction: column;
    gap: 24px;
  }
  section { display: flex; flex-direction: column; gap: 8px; }
  h2 {
    font-size: 13px;
    color: var(--text-primary);
    border-bottom: 1px solid var(--border);
    padding-bottom: 4px;
  }
  .desc { font-size: 12px; color: var(--text-secondary); }
  .row { display: flex; align-items: center; gap: 8px; }
  .hotkey-input {
    flex: 1;
    font-size: 13px;
    padding: 4px 8px;
  }
  .hint { font-size: 11px; color: var(--text-secondary); opacity: 0.7; }
  .toggle { display: flex; align-items: center; gap: 8px; cursor: pointer; }
  .toggle input[type="checkbox"] { width: 16px; height: 16px; cursor: pointer; accent-color: var(--accent); }
  .toggle-label { font-size: 13px; color: var(--text-primary); }
  .message { font-size: 12px; color: var(--accent); }
</style>
