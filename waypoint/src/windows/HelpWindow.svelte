<script lang="ts">
  import { onMount } from "svelte";
  import { windows as windowsApi, config as configApi } from "../lib/api";
  import DraggableTitlebar from "./DraggableTitlebar.svelte";
  import pkg from "../../package.json";
  const version = pkg.version;

  // 顯示當前快捷鍵；設定改完後，下次重新開啟使用說明會載入新值。
  let listHotkey = "Ctrl+Shift+Space";
  let passthroughHotkey = "Ctrl+Shift+G";

  onMount(async () => {
    try {
      const cfg = await configApi.get();
      listHotkey = cfg.hotkey || listHotkey;
      passthroughHotkey = cfg.passthroughHotkey || passthroughHotkey;
    } catch {
      // bg preview / 測試環境沒有 Tauri IPC 時保留預設值
    }
  });
</script>

<div class="help-window">
  <DraggableTitlebar label="help">
    <span class="title">Waypoint — 使用說明 <span class="version">v{version}</span></span>
    <button on:click={() => windowsApi.closeWindow("help").catch(() => {})}>✕</button>
  </DraggableTitlebar>

  <div class="content">
    <section>
      <h2>快捷鍵</h2>
      <table>
        <thead><tr><th>快捷鍵</th><th>功能</th><th>預設</th></tr></thead>
        <tbody>
          <tr>
            <td><strong>呼叫列表</strong></td>
            <td>列表沒開時 → 開列表（並還原上次的筆記）；列表開著時 → 儲存 session 並收起全部</td>
            <td><code>{listHotkey}</code></td>
          </tr>
          <tr>
            <td><strong>切換穿透</strong></td>
            <td>所有筆記同時切換滑鼠穿透：開啟 → 點得到、關掉 → 滑鼠/鍵盤穿過筆記到底層應用程式（適合邊看筆記邊操作其他軟體）</td>
            <td><code>{passthroughHotkey}</code></td>
          </tr>
        </tbody>
      </table>
      <p class="hint">兩個快捷鍵都可在列表視窗的設定（⚙）中重設。</p>
    </section>

    <section>
      <h2>全域筆記 vs 區域筆記</h2>
      <ul>
        <li><strong>全域筆記：</strong>無論在哪個應用程式按快捷鍵都會出現在列表中</li>
        <li><strong>區域筆記：</strong>只屬於特定應用程式（依 process 名稱或視窗標題識別）</li>
      </ul>
    </section>

    <section>
      <h2>✕ vs ⇊ 的差異</h2>
      <ul>
        <li><strong>✕（關閉按鈕）：</strong>永久關閉此筆記，下次不會自動還原</li>
        <li><strong>⇊（收起按鈕）：</strong>收起所有視窗，但記住哪些筆記是開著的，下次自動還原</li>
      </ul>
    </section>

    <section>
      <h2>視窗透明度</h2>
      <p>每個筆記 titlebar 內有獨立的透明度滑桿（10–100%），可讓視窗整體半透明，方便邊看筆記邊操作下方應用程式。設定值會儲存到該筆記的 settings，下次開啟自動套用。</p>
    </section>

    <section>
      <h2>如何設定 Context 識別方式</h2>
      <p>在列表視窗中，對 context 標題按右鍵，可選擇：程序名稱 或 視窗標題</p>
    </section>

    <section>
      <h2>跨平台 Context 對應</h2>
      <p>右鍵選單 →「對應到現有 context...」可將不同 OS 上的同一軟體合併。</p>
    </section>

    <section>
      <h2>資料夾位置</h2>
      <p>所有筆記和設定存放在：<code>~/waypoint/</code>（Windows: <code>C:\Users\&lt;user&gt;\waypoint\</code>）</p>
      <p>複製此資料夾到其他電腦即可使用相同筆記與設定。</p>
    </section>
  </div>
</div>

<style>
  .help-window { display: flex; flex-direction: column; height: 100vh; background: var(--bg-primary); }
  .title { font-size: 12px; font-weight: bold; color: var(--text-primary); pointer-events: none; }
  .version { font-weight: normal; color: var(--text-secondary); margin-left: 6px; font-size: 11px; }
  .content { flex: 1; overflow-y: auto; padding: 20px 24px; display: flex; flex-direction: column; gap: 24px; }
  section { display: flex; flex-direction: column; gap: 8px; }
  h2 { font-size: 13px; color: var(--text-primary); border-bottom: 1px solid var(--border); padding-bottom: 4px; }
  p { font-size: 12px; color: var(--text-secondary); line-height: 1.7; }
  ul { padding-left: 16px; display: flex; flex-direction: column; gap: 4px; }
  li { font-size: 12px; color: var(--text-secondary); line-height: 1.6; }
  code { background: var(--bg-tertiary); padding: 1px 5px; border-radius: 2px; color: var(--text-link); }
  .hint { font-size: 11px; color: var(--text-secondary); font-style: italic; }
  table { width: 100%; border-collapse: collapse; font-size: 12px; }
  th { text-align: left; padding: 5px 8px; background: var(--bg-tertiary); color: var(--text-secondary); border: 1px solid var(--border); }
  td { padding: 5px 8px; color: var(--text-primary); border: 1px solid var(--border); }
</style>
