<script lang="ts">
  import { onMount } from "svelte";
  import { config as configApi, passthrough, windows as windowsApi } from "../lib/api";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  // R12 fallback：data-tauri-drag-region 在 WebView2 上偶爾失效，
  // 補一個 mousedown handler 直接呼叫 startDragging，跳過 button/input。
  async function handleTitlebarMousedown(e: MouseEvent) {
    const target = e.target as HTMLElement | null;
    if (!target) return;
    if (target.closest("button, input, textarea, select, a")) return;
    try {
      await getCurrentWindow().startDragging();
    } catch {
      /* 在瀏覽器 mock 環境忽略 */
    }
  }

  let hotkey = "";
  let hotkeyInput = "";
  let autostartEnabled = false;
  let autostartSupported = false;
  let saving = false;
  let message = "";
  let capturing = false;

  // 穿透快捷鍵
  let passthroughHotkey = "";
  let passthroughHotkeyInput = "";
  let capturingPassthrough = false;

  // 把 KeyboardEvent 轉成 Tauri global shortcut 格式：Ctrl+Shift+Space / Alt+F1 ...
  function formatShortcut(e: KeyboardEvent): string | null {
    const parts: string[] = [];
    if (e.ctrlKey) parts.push("Ctrl");
    if (e.altKey) parts.push("Alt");
    if (e.shiftKey) parts.push("Shift");
    if (e.metaKey) parts.push("Super");

    const key = e.key;
    // 過濾單獨修飾鍵：使用者還在組鍵
    if (["Control", "Shift", "Alt", "Meta"].includes(key)) return null;

    let keyName: string;
    if (key === " ") keyName = "Space";
    else if (key.startsWith("Arrow")) keyName = key.slice(5); // ArrowUp → Up
    else if (key.length === 1) keyName = key.toUpperCase();
    else keyName = key; // F1..F12、Enter、Tab、Escape 等

    parts.push(keyName);
    return parts.join("+");
  }

  function startCapture() {
    capturing = true;
    hotkeyInput = "按下快捷鍵…";
  }

  function handleCapture(e: KeyboardEvent) {
    if (!capturing) return;
    e.preventDefault();
    e.stopPropagation();
    if (e.key === "Escape") {
      capturing = false;
      hotkeyInput = hotkey;
      return;
    }
    const combo = formatShortcut(e);
    if (combo) {
      hotkeyInput = combo;
      capturing = false;
    }
  }

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
    passthroughHotkey = cfg.passthroughHotkey ?? "";
    passthroughHotkeyInput = passthroughHotkey;
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

  function startCapturePassthrough() {
    capturingPassthrough = true;
    passthroughHotkeyInput = "按下快捷鍵…";
  }

  function handleCapturePassthrough(e: KeyboardEvent) {
    if (!capturingPassthrough) return;
    e.preventDefault();
    e.stopPropagation();
    if (e.key === "Escape") {
      capturingPassthrough = false;
      passthroughHotkeyInput = passthroughHotkey;
      return;
    }
    const combo = formatShortcut(e);
    if (combo) {
      passthroughHotkeyInput = combo;
      capturingPassthrough = false;
    }
  }

  async function savePassthroughHotkey() {
    if (!passthroughHotkeyInput.trim()) return;
    saving = true;
    try {
      await configApi.setPassthroughHotkey(passthroughHotkeyInput.trim());
      passthroughHotkey = passthroughHotkeyInput.trim();
      message = "穿透快捷鍵已儲存，重新啟動後生效";
    } catch (e) {
      message = `儲存失敗：${e}`;
    } finally {
      saving = false;
    }
  }
</script>

<div class="settings-window">
  <div class="titlebar" data-tauri-drag-region on:mousedown={handleTitlebarMousedown}>
    <span class="title">Waypoint — 設定</span>
    <button class="close-btn" on:click={() => windowsApi.closeWindow("settings").catch(() => {})}>✕</button>
  </div>

  <div class="content">
    <section>
      <h2>全域快捷鍵</h2>
      <p class="desc">按下快捷鍵開啟／收起 Waypoint 列表</p>
      <div class="row">
        <button
          type="button"
          class="hotkey-capture"
          class:capturing
          on:click={startCapture}
          on:keydown={handleCapture}
          title="點擊後按下要設定的快捷鍵"
        >
          {hotkeyInput || "點擊後按下快捷鍵"}
        </button>
        <button on:click={saveHotkey} disabled={saving || hotkeyInput === hotkey || capturing}>
          {saving ? "儲存中…" : "套用"}
        </button>
      </div>
      <p class="hint">點擊左側框後按下鍵盤組合鍵（Esc 取消）；範例：Ctrl+Shift+Space、Alt+F1</p>
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

    <section>
      <h2>穿透模式快捷鍵</h2>
      <p class="desc">按下後切換所有筆記穿透狀態</p>
      <div class="row">
        <button
          type="button"
          class="hotkey-capture"
          class:capturing={capturingPassthrough}
          on:click={startCapturePassthrough}
          on:keydown={handleCapturePassthrough}
          title="點擊後按下要設定的快捷鍵"
        >
          {passthroughHotkeyInput || "點擊後按下快捷鍵"}
        </button>
        <button on:click={savePassthroughHotkey} disabled={saving || passthroughHotkeyInput === passthroughHotkey || capturingPassthrough}>
          {saving ? "儲存中…" : "套用"}
        </button>
      </div>
      <p class="hint">點擊左側框後按下鍵盤組合鍵（Esc 取消）；預設：Ctrl+Shift+T</p>
    </section>

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
  .hotkey-capture {
    flex: 1;
    font-size: 13px;
    padding: 6px 10px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--text-primary);
    text-align: left;
    cursor: pointer;
  }
  .hotkey-capture:hover { border-color: var(--accent); }
  .hotkey-capture.capturing {
    border-color: var(--accent);
    color: var(--text-link);
    background: var(--bg-selected);
  }
  .hint { font-size: 11px; color: var(--text-secondary); opacity: 0.7; }
  .toggle { display: flex; align-items: center; gap: 8px; cursor: pointer; }
  .toggle input[type="checkbox"] { width: 16px; height: 16px; cursor: pointer; accent-color: var(--accent); }
  .toggle-label { font-size: 13px; color: var(--text-primary); }
  .message { font-size: 12px; color: var(--accent); }
</style>
