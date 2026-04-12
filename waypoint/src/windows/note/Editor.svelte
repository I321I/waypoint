<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from "svelte";
  import { Editor } from "@tiptap/core";
  import StarterKit from "@tiptap/starter-kit";
  import Underline from "@tiptap/extension-underline";
  import { TaskList } from "@tiptap/extension-task-list";
  import { TaskItem } from "@tiptap/extension-task-item";
  import { Markdown } from "@tiptap/markdown";
  import CodeBlockLowlight from "@tiptap/extension-code-block-lowlight";
  import { common, createLowlight } from "lowlight";

  export let content: string = "";
  export let fontSize: number = 14;

  const dispatch = createEventDispatcher<{ update: { markdown: string } }>();

  let element: HTMLElement;
  let editor: Editor;

  export function getEditor() { return editor; }

  const lowlight = createLowlight(common);

  onMount(() => {
    editor = new Editor({
      element,
      extensions: [
        StarterKit.configure({ codeBlock: false }),
        Underline,
        TaskList,
        TaskItem.configure({ nested: true }),
        CodeBlockLowlight.configure({ lowlight }),
        Markdown.configure({ transformPastedText: true, transformCopiedText: true }),
      ],
      editorProps: {
        attributes: {
          class: "tiptap-editor",
          style: `font-size: ${fontSize}px`,
        },
      },
      onUpdate({ editor }) {
        const markdown = (editor.storage as any).markdown?.getMarkdown?.() ?? "";
        dispatch("update", { markdown });
      },
    });

    if (content) {
      editor.commands.setContent(content);
    }
  });

  onDestroy(() => {
    editor?.destroy();
  });

  $: if (editor && editor.view?.dom) {
    (editor.view.dom as HTMLElement).style.fontSize = `${fontSize}px`;
  }
</script>

<div class="editor-wrap" bind:this={element} />

<style>
  .editor-wrap {
    flex: 1;
    overflow-y: auto;
    padding: 12px 14px;
    cursor: text;
  }
  :global(.tiptap-editor) {
    outline: none;
    min-height: 100%;
    color: var(--text-primary);
    font-family: var(--font-mono);
    line-height: 1.7;
  }
  :global(.tiptap-editor h1) { font-size: 1.5em; margin-bottom: 0.5em; color: var(--text-primary); }
  :global(.tiptap-editor h2) { font-size: 1.25em; margin-bottom: 0.4em; }
  :global(.tiptap-editor h3) { font-size: 1.1em; margin-bottom: 0.3em; }
  :global(.tiptap-editor ul, .tiptap-editor ol) { padding-left: 1.5em; }
  :global(.tiptap-editor li) { margin: 2px 0; }
  :global(.tiptap-editor code) { background: var(--bg-tertiary); padding: 1px 4px; border-radius: 2px; font-size: 0.9em; }
  :global(.tiptap-editor pre) { background: var(--bg-tertiary); padding: 10px 12px; border-radius: 3px; margin: 8px 0; overflow-x: auto; }
  :global(.tiptap-editor input[type="checkbox"]) { margin-right: 6px; accent-color: var(--accent); }
  :global(.tiptap-editor p) { margin: 4px 0; }
  :global(.tiptap-editor strong) { color: var(--text-primary); }
</style>
