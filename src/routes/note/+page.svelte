<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { invoke } from "@tauri-apps/api/core";

  let textarea: HTMLTextAreaElement;
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
    root.setProperty("--q-border-radius", `${t.border_radius}px`);
    root.setProperty("--q-border-color", t.border_color);
    root.setProperty("--q-border-thickness", `${t.border_thickness}px`);
  }

  function handleInput() {
    clearTimeout(saveTimeout);
    saveTimeout = setTimeout(() => {
      invoke("write_note", { content: textarea.value });
    }, 500);
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
    textarea.value = await invoke<string>("read_note");
    textarea.focus();
  });
</script>

<svelte:window on:keydown={handleKeydown} />

<main class="container">
  <div class="drag-region" data-tauri-drag-region></div>
  <!-- svelte-ignore a11y_autofocus -->
  <textarea
    bind:this={textarea}
    on:input={handleInput}
    placeholder="..."
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
  }

  .container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background-color: rgba(60, 60, 60, 1);
    overflow: hidden;
    box-sizing: border-box;
  }

  .drag-region {
    height: 24px;
    flex-shrink: 0;
    cursor: grab;
  }

  textarea {
    flex: 1;
    width: 100%;
    box-sizing: border-box;
    background: none;
    border: none;
    outline: none;
    resize: none;
    padding: 4px 16px 16px 16px;
    color: var(--q-font-color, #ffffff);
    font-size: var(--q-font-size, 14px);
    font-family: "JetBrainsMono Nerd Font", "Fira Code", "Cascadia Code", monospace;
    line-height: 1.6;
    caret-color: var(--q-font-color, #ffffff);
  }

  textarea::placeholder {
    color: rgba(255, 255, 255, 0.2);
  }
</style>
