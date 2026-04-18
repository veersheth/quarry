<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { invoke } from "@tauri-apps/api/core";
  import { fly } from "svelte/transition";
  import { backOut } from "svelte/easing";
  import RenderList from "$lib/RenderList.svelte";
  import RenderEmojis from "$lib/RenderEmojis.svelte";
  import ContextMenu from "$lib/ContextMenu.svelte";
  import {
    query,
    resultItems,
    resultType,
    activeIndex,
    contextMenu,
    openContextMenu,
    closeContextMenu,
  } from "../stores/search";
  import { search } from "../lib/searcher";
  import { handleKeydown } from "../lib/keyHandler";
  import { toasts } from "../stores/toasts";
  import RenderClipboard from "$lib/RenderClipboard.svelte";
  import RenderColorPicker from "$lib/RenderColorPicker.svelte";
  import RenderMath from "$lib/RenderMath.svelte";
  import RenderCamera from "$lib/RenderCamera.svelte";
  import RenderMarkdown from "$lib/RenderMarkdown.svelte";
  import RenderAiChat from "$lib/RenderAiChat.svelte";

  let searchInput: HTMLInputElement;
  let appWindow: ReturnType<typeof getCurrentWindow>;
  let isLoading = false;
  let searchTimeout: ReturnType<typeof setTimeout>;
  let resultsEl: HTMLDivElement;

  function handleContextMenu(event: MouseEvent, item: import("../stores/search").ResultItem) {
    if (item.actions.length <= 1) return;
    event.preventDefault();
    openContextMenu(item, event.clientX, event.clientY);
  }

  function handleOpenAtActive() {
    const items = $resultItems;
    const idx = $activeIndex;
    const item = items[idx];
    if (!item || item.actions.length <= 1) return;

    const activeRow = resultsEl?.querySelector("[data-active='true']") as HTMLElement | null;
    if (activeRow) {
      const rect = activeRow.getBoundingClientRect();
      openContextMenu(item, rect.left, rect.bottom + 4);
    } else {
      openContextMenu(item, window.innerWidth / 2, window.innerHeight / 2);
    }
  }

  interface Theme {
    background_color: string;
    background_opacity: number;
    font_size: number;
    font_color: string;
    border_radius: number;
    border_color: string;
    border_thickness: number;
    item_border_radius: number;
    active_bg_color: string;
    active_border_color: string;
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
    root.setProperty("--q-item-border-radius", `${t.item_border_radius}px`);
    root.setProperty("--q-active-bg-color", t.active_bg_color);
    root.setProperty("--q-active-border-color", t.active_border_color);
  }

  function forceRepaint() {
    // since gnome sometimes doesn't apply scoped styles on launch. this is a fix
    document.body.style.display = "none";
    void document.body.offsetHeight;
    document.body.style.display = "";
  }

  async function refresh() {
    const theme = await invoke<Theme>("get_theme");
    applyTheme(theme);
    requestAnimationFrame(forceRepaint);
    searchInput?.select();
  }

  onMount(async () => {
    forceRepaint();
    appWindow = getCurrentWindow();
    await refresh();
    const unlisten = appWindow.onFocusChanged(({ payload: focused }) => {
      if (focused) refresh();
    });

    window.addEventListener("open-context-menu-at-active", handleOpenAtActive);

    return () => {
      unlisten.then((fn) => fn());
      window.removeEventListener("open-context-menu-at-active", handleOpenAtActive);
    };
  });

  $: if ($query !== undefined) {
    isLoading = true;
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => {
      search($query)
        .then((res) => {
          if (res === null) return;
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
    <div class="results" class:loading-overlay={isLoading} bind:this={resultsEl}>
      <div class="results-content" class:dimmed={isLoading}>
        {#if $resultType === "List"}
          <RenderList listitems={$resultItems} {activeIndex} onContextMenu={handleContextMenu} />
        {:else if $resultType === "Grid"}
          <RenderEmojis listitems={$resultItems} {activeIndex} onContextMenu={handleContextMenu} />
        {:else if $resultType === "Markdown"}
          <RenderMarkdown listitems={$resultItems} {activeIndex} />
        {:else if $resultType === "Clipboard"}
          <RenderClipboard listitems={$resultItems} {activeIndex} onContextMenu={handleContextMenu} />
        {:else if $resultType === "ColorPicker"}
          <RenderColorPicker />
        {:else if $resultType === "Home"}
          <RenderList listitems={$resultItems} {activeIndex} onContextMenu={handleContextMenu} />
        {:else if $resultType === "Math"}
          <RenderMath listitems={$resultItems} {activeIndex} onContextMenu={handleContextMenu} />
        {:else if $resultType === "Camera"}
          <RenderCamera />
        {:else if $resultType === "Ai"}
          <RenderAiChat />
        {:else}
          Oops
        {/if}
      </div>
    </div>
  </div>

  {#if $contextMenu.open && $contextMenu.item}
    <ContextMenu
      item={$contextMenu.item}
      x={$contextMenu.x}
      y={$contextMenu.y}
      onClose={closeContextMenu}
    />
  {/if}

  <!-- Toasts -->
  <div class="toast-container">
    {#each $toasts as toast (toast.id)}
      <div
        class="toast {toast.type}"
        in:fly={{ y: 16, duration: 350, opacity: 0, easing: backOut }}
        out:fly={{ y: 8, duration: 140, opacity: 0 }}
      >
        <!-- svelte-ignore element_invalid_self_closing_tag -->
        <span class="toast-dot {toast.type}" />
        {toast.message}
      </div>
    {/each}
  </div>
</main>

<style>
  .container {
    display: flex;
    flex: 1;
    height: calc(100vh - 2px);
    flex-direction: column;
    margin: 0;
    padding: 0;
    box-sizing: border-box;
    background-color: var(--q-bg-color, rgba(15, 15, 15, 1));
    opacity: var(--q-bg-opacity, 1);
    overflow: hidden;
    color: var(--q-font-color, #ffffff);
    position: relative;
  }

  .container * {
    z-index: 0;
    color: var(--q-font-color, #ffffff);
    font-size: var(--q-font-size);
    font-family:
      "Inter",
      "Segoe UI",
      "Adwaita Sans",
      "Noto Color Emoji",
      sans-serif;
    padding: 0;
  }

  .panel {
    display: flex;
    flex-direction: column;
    flex: 1;
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

  /* Toasts */
  .toast-container {
    position: absolute;
    bottom: 14px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    flex-direction: column;
    gap: 6px;
    align-items: center;
    pointer-events: none;
    z-index: 1000;
  }

  .toast {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 20px 30px 25px;
    border-radius: 999px;
    font-size: 0.95rem;
    letter-spacing: 0.01em;
    white-space: nowrap;
    z-index: 1000;
    border: 1px solid rgba(255, 255, 255, 0.18);
    background: rgba(40, 40, 40, 0.70);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    box-shadow: 0 4px 24px rgba(0, 0, 0, 0.4);
    color: rgba(255, 255, 255, 0.75);
  }

  .toast-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .toast-dot.success { background: #4ade80; }
  .toast-dot.error   { background: #f87171; }
  .toast-dot.info    { background: #60a5fa; }

  .toast.error {
    border-color: rgba(248, 113, 113, 0.2);
  }

  .toast.info {
    border-color: rgba(96, 165, 250, 0.2);
  }
</style>
