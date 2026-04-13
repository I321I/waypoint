<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import ListWindowComp from "../windows/ListWindow.svelte";
  import NoteWindowComp from "../windows/NoteWindow.svelte";
  import HelpWindowComp from "../windows/HelpWindow.svelte";
  import SettingsWindowComp from "../windows/SettingsWindow.svelte";

  // 用視窗 label 決定顯示哪個元件——比 URL 解析更可靠（Windows WebView2 可能讀不到 search params）
  // list        → ListWindow
  // note-{id}   → NoteWindow
  // help        → HelpWindow
  // settings    → SettingsWindow
  const label = getCurrentWindow().label;
  const isNote = label.startsWith("note-");

  // noteId 從 label 取，避免 URL 讀不到
  const noteId = isNote ? label.slice(5) : null;
  // contextId 仍用 URL（note 視窗的附加參數；若讀不到就當 global note）
  const contextId = isNote
    ? new URLSearchParams(window.location.search).get("contextId")
    : null;
</script>

{#if label === "list"}
  <ListWindowComp />
{:else if isNote && noteId}
  <NoteWindowComp {noteId} {contextId} />
{:else if label === "help"}
  <HelpWindowComp />
{:else if label === "settings"}
  <SettingsWindowComp />
{/if}
