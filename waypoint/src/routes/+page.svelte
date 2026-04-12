<script lang="ts">
  import { onMount } from "svelte";
  import type { ViewType } from "$lib/types";

  let view: ViewType = "list";
  let noteId: string | null = null;
  let contextId: string | null = null;
  let ListWindow: any;
  let NoteWindow: any;
  let HelpWindow: any;

  onMount(async () => {
    const params = new URLSearchParams(window.location.search);
    view = (params.get("view") as ViewType) ?? "list";
    noteId = params.get("noteId");
    contextId = params.get("contextId");

    // Lazy load the appropriate window component
    if (view === "list") {
      const mod = await import("../windows/ListWindow.svelte");
      ListWindow = mod.default;
    } else if (view === "note") {
      const mod = await import("../windows/NoteWindow.svelte");
      NoteWindow = mod.default;
    } else if (view === "help") {
      const mod = await import("../windows/HelpWindow.svelte");
      HelpWindow = mod.default;
    }
  });
</script>

{#if view === "list" && ListWindow}
  <svelte:component this={ListWindow} />
{:else if view === "note" && NoteWindow && noteId}
  <svelte:component this={NoteWindow} {noteId} {contextId} />
{:else if view === "help" && HelpWindow}
  <svelte:component this={HelpWindow} />
{/if}
