<script lang="ts">
  import type { ViewType } from "$lib/types";
  import ListWindowComp from "../windows/ListWindow.svelte";
  import NoteWindowComp from "../windows/NoteWindow.svelte";
  import HelpWindowComp from "../windows/HelpWindow.svelte";
  import SettingsWindowComp from "../windows/SettingsWindow.svelte";

  // URL hash 路由：#view=list / #view=help / #view=settings / #view=note&noteId=xxx&contextId=yyy
  // hash 不會被 SvelteKit router 修改，也不依賴 __TAURI_INTERNALS__，跨平台可靠
  const hashStr = window.location.hash.startsWith('#') ? window.location.hash.slice(1) : '';
  const params = new URLSearchParams(hashStr);
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
