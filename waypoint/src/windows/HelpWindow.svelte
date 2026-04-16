<script lang="ts">
  import { windows as windowsApi } from "../lib/api";
</script>

<div class="help-window">
  <div class="titlebar" data-tauri-drag-region>
    <span class="title">Waypoint — 使用說明</span>
    <button on:click={() => windowsApi.closeWindow("help").catch(() => {})}>✕</button>
  </div>

  <div class="content">
    <section>
      <h2>快捷鍵邏輯（三段式）</h2>
      <table>
        <thead><tr><th>狀態</th><th>按快捷鍵</th><th>結果</th></tr></thead>
        <tbody>
          <tr><td>無任何視窗</td><td>按一次</td><td>開啟列表 + 還原上次筆記</td></tr>
          <tr><td>有筆記，無列表</td><td>按一次</td><td>開啟列表</td></tr>
          <tr><td>列表開著</td><td>再按一次</td><td>儲存 session，收起全部</td></tr>
        </tbody>
      </table>
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
      <h2>如何設定 Context 識別方式</h2>
      <p>在列表視窗中，對 context 標題按右鍵，可選擇：程序名稱 或 視窗標題</p>
    </section>

    <section>
      <h2>跨平台 Context 對應</h2>
      <p>右鍵選單 →「對應到現有 context...」可將不同 OS 上的同一軟體合併。</p>
    </section>

    <section>
      <h2>資料夾位置</h2>
      <p>所有筆記和設定存放在：<code>~/waypoint/</code></p>
      <p>複製此資料夾到其他電腦即可使用相同筆記與設定。</p>
    </section>
  </div>
</div>

<style>
  .help-window { display: flex; flex-direction: column; height: 100vh; background: var(--bg-primary); }
  .titlebar {
    display: flex; align-items: center; justify-content: space-between;
    padding: 6px 12px; background: var(--bg-tertiary); border-bottom: 1px solid var(--border);
  }
  .title { font-size: 12px; font-weight: bold; color: var(--text-primary); }
  .content { flex: 1; overflow-y: auto; padding: 20px 24px; display: flex; flex-direction: column; gap: 24px; }
  section { display: flex; flex-direction: column; gap: 8px; }
  h2 { font-size: 13px; color: var(--text-primary); border-bottom: 1px solid var(--border); padding-bottom: 4px; }
  p { font-size: 12px; color: var(--text-secondary); line-height: 1.7; }
  ul { padding-left: 16px; display: flex; flex-direction: column; gap: 4px; }
  li { font-size: 12px; color: var(--text-secondary); line-height: 1.6; }
  code { background: var(--bg-tertiary); padding: 1px 5px; border-radius: 2px; color: var(--text-link); }
  table { width: 100%; border-collapse: collapse; font-size: 12px; }
  th { text-align: left; padding: 5px 8px; background: var(--bg-tertiary); color: var(--text-secondary); border: 1px solid var(--border); }
  td { padding: 5px 8px; color: var(--text-primary); border: 1px solid var(--border); }
</style>
