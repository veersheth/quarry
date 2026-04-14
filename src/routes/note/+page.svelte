<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { invoke } from "@tauri-apps/api/core";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import ink from "ink-mde";

  let editorEl: HTMLDivElement;
  let saveTimeout: ReturnType<typeof setTimeout>;
  let editor: ReturnType<typeof ink> | null = null;

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

  function handleGlobalKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
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

    const imageTypes = ["image/png", "image/jpeg", "image/gif", "image/webp", "image/svg+xml"];
    for (const file of Array.from(files)) {
      if (!imageTypes.includes(file.type)) continue;
      // In Tauri, dropped files have a real path accessible via the webview
      const path = (file as any).path as string | undefined;
      const url = path ? convertFileSrc(path) : URL.createObjectURL(file);
      insertMarkdownAtCursor(`![${file.name}](${url})\n`);
    }
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
  }

  onMount(async () => {
    const [theme, saved] = await Promise.all([
      invoke<Theme>("get_theme"),
      invoke<string>("read_note"),
    ]);

    applyTheme(theme);

    editor = ink(editorEl, {
      doc: saved,
      hooks: {
        beforeUpdate(doc: string) {
          scheduleSave(doc);
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
      vim: false,
    });

    editorEl.querySelector<HTMLElement>(".cm-content")?.focus();
  });

  onDestroy(() => {
    clearTimeout(saveTimeout);
    editor?.destroy();
  });
</script>

<svelte:window on:keydown={handleGlobalKeydown} />

<main class="container" on:drop={handleDrop} on:dragover={handleDragOver}>
  <div class="editor-wrap" bind:this={editorEl}></div>
  <div class="titlebar" data-tauri-drag-region>
    <span class="title">QUARRY NOTEPAD</span>
  </div>
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
    background-color: #1a1a1f;
    overflow: hidden;
    box-sizing: border-box;
    font-family: "JetBrainsMono Nerd Font", monospace;
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
    font-family:
      Inter,
      "Segoe UI",
      "Adwaita Sans",
      "Noto Color Emoji",
      sans-serif;
    font-size: var(--q-font-size, 16px);
  }

  :global(.editor-wrap .cm-scroller) {
    overflow-y: auto !important;
    padding: 20px;
    box-sizing: border-box;
    line-height: 1.7;
  }

  :global(.editor-wrap .cm-content) {
    color: var(--q-font-color, rgba(226, 219, 197, 0.88));
    white-space: pre-wrap;
    word-break: break-word;
  }

  :global(.editor-wrap img) {
    max-width: 100%;
    border-radius: 4px;
    margin: 4px 0;
    display: block;
  }

  .titlebar {
    display: flex;
    align-items: center;
    height: 36px;
    padding: 0 14px;
    flex-shrink: 0;
    border-top: 1px solid rgba(255, 255, 255, 0.13);
    background: rgba(255, 255, 255, 0.03);
    user-select: none;
    cursor: grab;
  }

  .title {
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.08em;
    color: rgba(255, 255, 255, 0.2);
    user-select: none;
  }
</style>
