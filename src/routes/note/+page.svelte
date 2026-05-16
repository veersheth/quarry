<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { invoke } from "@tauri-apps/api/core";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import ink from "ink-mde";

  let editorEl: HTMLDivElement;
  let saveTimeout: ReturnType<typeof setTimeout>;
  let editor: ReturnType<typeof ink> | null = null;
  let vimMode = false;
  let docContent = "";
  let uiScale = 1.0;

  interface Theme {
    background_color: string;
    background_opacity: number;
    font_size: number;
    font_color: string;
    border_radius: number;
    border_color: string;
    border_thickness: number;
  }

  function applyTheme(t: Theme) {
    const root = document.documentElement.style;
    root.setProperty("--q-bg-color", t.background_color);
    root.setProperty("--q-bg-opacity", String(t.background_opacity));
    root.setProperty("--q-font-size", `${t.font_size}px`);
    root.setProperty("--q-font-color", t.font_color);
    root.setProperty("--q-border-color", t.border_color);
    root.setProperty("--q-border-thickness", `${t.border_thickness}px`);
  }

  function scheduleSave(value: string) {
    clearTimeout(saveTimeout);
    saveTimeout = setTimeout(() => {
      invoke("write_note", { content: value });
    }, 400);
  }

  function createEditor(doc: string) {
    editor?.destroy();
    // Clear any leftover CodeMirror DOM so ink starts fresh
    editorEl.innerHTML = "";
    editor = ink(editorEl, {
      doc,
      hooks: {
        beforeUpdate(value: string) {
          docContent = value;
          scheduleSave(value);
        },
      },
      interface: {
        appearance: "dark",
        attribution: false,
        autocomplete: false,
        images: true,
        lists: true,
        readonly: false,
        spellcheck: false,
        toolbar: false,
      },
      vim: vimMode,
    });
    editorEl.querySelector<HTMLElement>(".cm-content")?.focus();
  }

  function toggleVim() {
    vimMode = !vimMode;
    localStorage.setItem("quarry-vim-mode", String(vimMode));
    createEditor(docContent);
  }

  function handleGlobalKeydown(e: KeyboardEvent) {
    if (e.ctrlKey && (e.key === "-" || e.key === "=" || e.key === "+" || e.key === "0")) {
      e.preventDefault();
      if (e.key === "-") uiScale = Math.max(0.6, parseFloat((uiScale - 0.05).toFixed(2)));
      else if (e.key === "=" || e.key === "+") uiScale = Math.min(2.0, parseFloat((uiScale + 0.05).toFixed(2)));
      else uiScale = 1.0;
      return;
    }
    if (e.key === "Escape" && !vimMode) {
      e.preventDefault();
      getCurrentWindow().hide();
    }
  }

  function insertMarkdownAtCursor(markdown: string) {
    if (!editor) return;
    const state = (editor as any).instance?.state;
    const pos = state?.selection?.main?.head ?? 0;
    (editor as any).instance?.dispatch({
      changes: { from: pos, insert: markdown },
      selection: { anchor: pos + markdown.length },
    });
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    const files = e.dataTransfer?.files;
    if (!files?.length || !editor) return;

    const imageTypes = [
      "image/png",
      "image/jpeg",
      "image/gif",
      "image/webp",
      "image/svg+xml",
    ];
    for (const file of Array.from(files)) {
      if (!imageTypes.includes(file.type)) continue;
      const path = (file as any).path as string | undefined;
      const url = path ? convertFileSrc(path) : URL.createObjectURL(file);
      insertMarkdownAtCursor(`![${file.name}](${url})\n`);
    }
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
  }

  onMount(async () => {
    vimMode = localStorage.getItem("quarry-vim-mode") === "true";

    const [theme, saved] = await Promise.all([
      invoke<Theme>("get_theme"),
      invoke<string>("read_note"),
    ]);

    applyTheme(theme);
    docContent = saved;
    createEditor(saved);
  });

  onDestroy(() => {
    clearTimeout(saveTimeout);
    editor?.destroy();
  });
</script>

<svelte:window on:keydown={handleGlobalKeydown} />

<main
  class="container"
  class:vim-mode={vimMode}
  style="zoom: {uiScale}"
  on:drop={handleDrop}
  on:dragover={handleDragOver}
>
  <div class="toolbar">
    <button
      class="vim-toggle"
      class:active={vimMode}
      on:click={toggleVim}
      title="Toggle Vim mode"
    >
      VIM
    </button>
  </div>
  <div class="editor-wrap" bind:this={editorEl}></div>
</main>

<style>
  :global(html, body) {
    margin: 0;
    padding: 0;
    height: 100%;
    overflow: hidden;
    background: transparent;
  }

  .container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background-color: #111114;
    color: white;
    overflow: hidden;
    box-sizing: border-box;
    font-family: "JetBrainsMono Nerd Font", "Fira Code", "Cascadia Mono",
      monospace;
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    padding: 7px 12px;
    flex-shrink: 0;
    border-bottom: 0.5px solid rgba(255, 255, 255, 0.06);
  }

  .vim-toggle {
    background: none;
    border: 0.5px solid rgba(255, 255, 255, 0.1);
    border-radius: 4px;
    color: rgba(255, 255, 255, 0.2);
    font-family: "JetBrainsMono Nerd Font", "Fira Code", "Cascadia Mono",
      monospace;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.1em;
    padding: 3px 8px;
    cursor: pointer;
    transition:
      color 0.1s,
      border-color 0.1s;
  }

  .vim-toggle:hover {
    color: rgba(255, 255, 255, 0.45);
    border-color: rgba(255, 255, 255, 0.2);
  }

  .vim-toggle.active {
    color: #7ec8a4;
    border-color: rgba(126, 200, 164, 0.35);
    background: rgba(126, 200, 164, 0.06);
  }

  .editor-wrap {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  :global(.editor-wrap .ink) {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  :global(.editor-wrap .cm-editor) {
    flex: 1;
    min-height: 0;
    font-family: Inter, "Segoe UI", "Adwaita Sans", sans-serif;
    font-size: var(--q-font-size, 15px);
  }

  :global(.editor-wrap .cm-scroller) {
    overflow-y: auto !important;
    box-sizing: border-box;
    padding: 1rem 1.75rem;
    line-height: 1.75;
  }

  :global(.editor-wrap .cm-content) {
    color: var(--q-font-color, rgba(226, 219, 197, 0.82));
    white-space: pre-wrap;
    word-break: break-word;
  }

  :global(.editor-wrap .cm-content ::selection),
  :global(.editor-wrap .cm-content *::selection) {
    background: rgba(100, 160, 255, 0.3) !important;
  }

  :global(.vim-mode .cm-selectionBackground),
  :global(.vim-mode .cm-focused .cm-selectionBackground) {
    background: rgba(100, 160, 255, 0.3) !important;
  }

  :global(.vim-mode .editor-wrap .cm-editor),
  :global(.vim-mode .editor-wrap .cm-content),
  :global(.vim-mode .editor-wrap .cm-line) {
    font-family: "JetBrainsMono Nerd Font", "Fira Code", "Cascadia Mono",
      monospace;
  }

  :global(.editor-wrap img) {
    max-width: 100%;
    border-radius: 4px;
    margin: 4px 0;
    display: block;
  }
</style>
