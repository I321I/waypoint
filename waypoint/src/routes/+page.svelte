<script lang="ts">
  import type { ViewType } from "$lib/types";
  import ListWindowComp from "../windows/ListWindow.svelte";
  import NoteWindowComp from "../windows/NoteWindow.svelte";
  import HelpWindowComp from "../windows/HelpWindow.svelte";
  import SettingsWindowComp from "../windows/SettingsWindow.svelte";

  // 同步讀取 URL params（ssr: false，window 永遠可用）
  // 避免 onMount 非同步讀取造成的競態白屏問題
  const params = new URLSearchParams(window.location.search);
  const view = (params.get("view") ?? "list") as ViewType;
  const noteId = params.get("noteId");
  const contextId = params.get("contextId");
</script>

{#if view === "list"}
  <ListWindowComp />
{:else if view === "note" && noteId}
  <NoteWindowComp {noteId} {contextId} />
{:else if view === "help"}
  <HelpWindowComp />
{:else if view === "settings"}
  <SettingsWindowComp />
{/if}
