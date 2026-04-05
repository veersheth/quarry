<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { invoke } from "@tauri-apps/api/core";
  import RenderList from "$lib/RenderList.svelte";
  import RenderEmojis from "$lib/RenderEmojis.svelte";
  import {
    query,
    resultItems,
    resultType,
    activeIndex,
  } from "../stores/search";
  import { search } from "../lib/searcher";
  import { handleKeydown } from "../lib/keyHandler";
  import RenderClipboard from "$lib/RenderClipboard.svelte";
  import RenderColorPicker from "$lib/RenderColorPicker.svelte";
  import RenderWebSearch from "$lib/RenderWebSearch.svelte";
  import RenderMath from "$lib/RenderMath.svelte";
  import RenderCamera from "$lib/RenderCamera.svelte";
  import RenderMarkdown from "$lib/RenderMarkdown.svelte";

  let searchInput: HTMLInputElement;
  let appWindow: ReturnType<typeof getCurrentWindow>;
  let isLoading = false;
  let searchTimeout: ReturnType<typeof setTimeout>;

  interface Theme {
    background_color:    string;
    background_opacity:  number;
    font_size: number;
    font_color:          string;
    border_radius:       number;
    border_color:        string;
    border_thickness:    number;
    item_border_radius:  number;
    active_bg_color:     string;
    active_border_color: string;
  }

  function applyTheme(t: Theme) {
    const root = document.documentElement.style;
    root.setProperty("--q-bg-color",           t.background_color);
    root.setProperty("--q-bg-opacity",         String(t.background_opacity));
    root.setProperty("--q-font-size",           `${t.font_size}px`);
    root.setProperty("--q-font-color",         t.font_color);
    root.setProperty("--q-border-radius",      `${t.border_radius}px`);
    root.setProperty("--q-border-color",       t.border_color);
    root.setProperty("--q-border-thickness",   `${t.border_thickness}px`);
    root.setProperty("--q-item-border-radius", `${t.item_border_radius}px`);
    root.setProperty("--q-active-bg-color",          t.active_bg_color);
    root.setProperty("--q-active-border-color",      t.active_border_color);
  }

  async function refresh() {
    const theme = await invoke<Theme>("get_theme");
    applyTheme(theme);
    searchInput?.focus();
  }

  onMount(async () => {
    appWindow = getCurrentWindow();
    await refresh(); // apply theme
    const unlisten = appWindow.onFocusChanged(({ payload: focused }) => {
      if (focused) refresh();
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  });

  $: if ($query !== undefined) {
    isLoading = true;
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => {
      search($query)
        .then((res) => {
          resultItems.set(res.results);
          resultType.set(res.result_type);
          activeIndex.set(0);
        })
        .catch((err) => {
          console.error("Search error:", err);
          resultItems.set([]);
        })
        .finally(() => {
          isLoading = false;
        });
    }, 100);
  }
</script>

<svelte:window
  on:keydown={(e) =>
    handleKeydown(e, searchInput, activeIndex, resultItems, appWindow)}
/>

<main class="container">
  <div class="panel">
    <!-- svelte-ignore a11y_autofocus -->
    <input
      type="text"
      placeholder="quarry..."
      bind:value={$query}
      bind:this={searchInput}
      autofocus
      class="search"
      class:loading={isLoading}
    />
    <div class="results" class:loading-overlay={isLoading}>
      <div class="results-content" class:dimmed={isLoading}>
        {#if $resultType === "List"}
          <RenderList listitems={$resultItems} {activeIndex} />
        {:else if $resultType === "Grid"}
          <RenderEmojis listitems={$resultItems} {activeIndex} />
        {:else if $resultType === "WebSearch"}
          <RenderWebSearch listitems={$resultItems} {activeIndex} />
        {:else if $resultType === "Markdown"}
          <RenderMarkdown listitems={$resultItems} {activeIndex} />
        {:else if $resultType === "Clipboard"}
          <RenderClipboard listitems={$resultItems} {activeIndex} />
        {:else if $resultType === "ColorPicker"}
          <RenderColorPicker />
        {:else if $resultType === "Home"}
          <RenderList listitems={$resultItems} {activeIndex} />
        {:else if $resultType === "Math"}
          <RenderMath listitems={$resultItems} {activeIndex} />
        {:else if $resultType === "Camera"}
          <RenderCamera />
        {:else}
          Oops
        {/if}
      </div>
    </div>
  </div>
</main>

<style>
  .container {
    display: flex;
    flex: 1;
    height: calc(100vh - 20px);
    flex-direction: column;
    margin: 0;
    padding: 0;
    box-sizing: border-box;
    background-color: var(--q-bg-color, rgba(15, 15, 15, 1));
    opacity: var(--q-bg-opacity, 1);
    border: var(--q-border-thickness, 1px) solid var(--q-border-color, rgba(255,255,255,0.35));
    overflow: hidden;
    border-radius: var(--q-border-radius, 14px);
    color: var(--q-font-color, #ffffff);
  }

  .container * {
    z-index: 0;
    color: var(--q-font-color, #ffffff);
    font-size: var(--q-font-size);
    font-family:
      Inter,
      "Segoe UI",
      "Adwaita Sans",
      "Noto Color Emoji",
      sans-serif;
  }

  .panel {
    display: flex;
    flex-direction: column;
    flex: 1;
    max-height: 95vh;
    min-height: 0;
  }

  .search {
    width: 100%;
    display: block;
    padding: 0 20px;
    margin: 0;
    box-sizing: border-box;
    border: none;
    outline: none;
    background: none;
    height: 56px;
    flex-shrink: 0;
    transition: opacity 0.15s ease;
  }

  .search.loading {
    opacity: 0.7;
  }

  .results {
    margin: 0;
    padding: 0;
    border-top: 1px solid rgba(80, 80, 80, 0.7);
    flex: 1;
    box-sizing: border-box;
    overflow-y: auto;
    position: relative;
    min-height: 0;
  }

  .results-content {
    transition: opacity 0.1s ease;
    height: 100%;
  }

  .results-content.dimmed {
    opacity: 0.5;
    pointer-events: none;
  }
</style>
