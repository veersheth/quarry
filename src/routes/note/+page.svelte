<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { invoke } from "@tauri-apps/api/core";

  let textarea: HTMLTextAreaElement;
  let content = "";
  let saveTimeout: ReturnType<typeof setTimeout>;
  let appWindow: ReturnType<typeof getCurrentWindow>;

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

  function handleInput() {
    clearTimeout(saveTimeout);
    saveTimeout = setTimeout(() => {
      invoke("write_note", { content });
    }, 400);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      appWindow.hide();
    }
  }

  onMount(async () => {
    appWindow = getCurrentWindow();
    const theme = await invoke<Theme>("get_theme");
    applyTheme(theme);
    content = await invoke<string>("read_note");
    textarea?.focus();
  });
</script>

<svelte:window on:keydown={handleKeydown} />

<main class="container">
  <div class="titlebar" data-tauri-drag-region>
    <span class="title">QUARRY NOTEPAD</span>
  </div>
  <!-- svelte-ignore a11y_autofocus -->
  <textarea
    bind:this={textarea}
    bind:value={content}
    on:input={handleInput}
    placeholder="Note note, note"
    spellcheck="false"
    autofocus
  ></textarea>
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
  }

  .titlebar {
    display: flex;
    align-items: center;
    height: 36px;
    padding: 0 14px;
    flex-shrink: 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    background: rgba(255, 255, 255, 0.02);
    user-select: none;
  }

  .title {
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.08em;
    color: rgba(255, 255, 255, 0.2);
    user-select: none;
  }

  textarea {
    flex: 1;
    width: 100%;
    box-sizing: border-box;
    background: none;
    border: none;
    outline: none;
    resize: none;
    padding: 14px 16px 16px 16px;
    color: var(--q-font-color, rgba(255, 255, 255, 0.88));
    font-size: var(--q-font-size, 13px);
    font-family: "JetBrainsMono Nerd Font", "Fira Code", "Cascadia Code", monospace;
    line-height: 1.7;
    caret-color: #7c9ef8;
  }

  textarea::placeholder {
    color: rgba(255, 255, 255, 0.12);
  }

  textarea::-webkit-scrollbar {
    width: 4px;
  }

  textarea::-webkit-scrollbar-track {
    background: transparent;
  }

  textarea::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
  }

  textarea::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.2);
  }
</style>

