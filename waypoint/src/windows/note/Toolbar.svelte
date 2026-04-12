<script lang="ts">
  import type { Editor } from "@tiptap/core";

  export let editor: Editor | null = null;
  export let onOpenSettings: () => void;

  function cmd(command: () => boolean) {
    if (!editor) return;
    command();
    editor.view.focus();
  }
</script>

<div class="toolbar">
  <button
    class:active={editor?.isActive("bold")}
    on:click={() => cmd(() => editor!.chain().focus().toggleBold().run())}
    title="Bold"><b>B</b></button>
  <button
    class:active={editor?.isActive("italic")}
    on:click={() => cmd(() => editor!.chain().focus().toggleItalic().run())}
    title="Italic"><i>I</i></button>
  <button
    class:active={editor?.isActive("underline")}
    on:click={() => cmd(() => editor!.chain().focus().toggleUnderline().run())}
    title="Underline"><u>U</u></button>
  <button
    class:active={editor?.isActive("strike")}
    on:click={() => cmd(() => editor!.chain().focus().toggleStrike().run())}
    title="Strikethrough"><s>S</s></button>
  <span class="sep" />
  <button
    class:active={editor?.isActive("heading", { level: 1 })}
    on:click={() => cmd(() => editor!.chain().focus().toggleHeading({ level: 1 }).run())}
    title="Heading 1">H1</button>
  <button
    class:active={editor?.isActive("heading", { level: 2 })}
    on:click={() => cmd(() => editor!.chain().focus().toggleHeading({ level: 2 }).run())}
    title="Heading 2">H2</button>
  <button
    class:active={editor?.isActive("heading", { level: 3 })}
    on:click={() => cmd(() => editor!.chain().focus().toggleHeading({ level: 3 }).run())}
    title="Heading 3">H3</button>
  <span class="sep" />
  <button
    class:active={editor?.isActive("bulletList")}
    on:click={() => cmd(() => editor!.chain().focus().toggleBulletList().run())}
    title="Bullet list">≡</button>
  <button
    class:active={editor?.isActive("taskList")}
    on:click={() => cmd(() => editor!.chain().focus().toggleTaskList().run())}
    title="Task list">☑</button>
  <button
    class:active={editor?.isActive("codeBlock")}
    on:click={() => cmd(() => editor!.chain().focus().toggleCodeBlock().run())}
    title="Code block">&lt;/&gt;</button>
  <span class="spacer" />
  <button class="settings-btn" on:click={onOpenSettings} title="設定">⚙</button>
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 3px 8px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    min-height: 28px;
  }
  .toolbar button { font-size: 11px; padding: 2px 5px; min-width: 22px; }
  .toolbar button.active { background: var(--bg-selected); color: var(--text-primary); }
  .sep { width: 1px; height: 14px; background: var(--border); margin: 0 3px; }
  .spacer { flex: 1; }
  .settings-btn { font-size: 13px; }
</style>
