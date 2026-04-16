<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { invoke } from "@tauri-apps/api/core";
  import { fly, scale } from "svelte/transition";
  import { backOut, cubicOut } from "svelte/easing";
  import RenderList from "$lib/RenderList.svelte";
  import RenderEmojis from "$lib/RenderEmojis.svelte";
  import {
    query,
    resultItems,
    resultType,
    activeIndex,
    contextMenu,
    openContextMenu,
    closeContextMenu,
  } from "../stores/search";
  import { search, execute } from "../lib/searcher";
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

  // Derive a label from an action_id for display in the dropdown.
  // e.g. "open_browser" -> "Open Browser"
// Position the context menu near the triggering element, keeping it in-bounds.
  function positionMenu(x: number, y: number) {
    // We'll clamp after rendering via a tick, but store raw coords now.
    contextMenu.update(s => ({ ...s, x, y }));
  }

  // Right-click on a result row
  function handleContextMenu(event: MouseEvent, item: import("../stores/search").ResultItem) {
    if (item.actions.length <= 1) return;
    event.preventDefault();
    openContextMenu(item, event.clientX, event.clientY);
  }

  // Ctrl+K from keyHandler fires this custom event so we can anchor to the active row
  function handleOpenAtActive() {
    const items = $resultItems;
    const idx = $activeIndex;
    const item = items[idx];
    if (!item || item.actions.length <= 1) return;

    // Try to find the active row element
    const activeRow = resultsEl?.querySelector("[data-active='true']") as HTMLElement | null;
    if (activeRow) {
      const rect = activeRow.getBoundingClientRect();
      openContextMenu(item, rect.right - 8, rect.top);
    } else {
      openContextMenu(item, window.innerWidth / 2, window.innerHeight / 2);
    }
  }

  // Clamp the menu position after it renders so it stays on screen
  let menuEl: HTMLDivElement | null = null;
  $: if ($contextMenu.open && menuEl) {
    // Run after paint
    requestAnimationFrame(() => {
      if (!menuEl) return;
      const rect = menuEl.getBoundingClientRect();
      const vw = window.innerWidth;
      const vh = window.innerHeight;
      let { x, y } = $contextMenu;
      if (x + rect.width > vw - 8) x = vw - rect.width - 8;
      if (y + rect.height > vh - 8) y = vh - rect.height - 8;
      if (x < 8) x = 8;
      if (y < 8) y = 8;
      contextMenu.update(s => ({ ...s, x, y }));
    });
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

  async function refresh() {
    const theme = await invoke<Theme>("get_theme");
    applyTheme(theme);
    searchInput?.select();
  }

  onMount(async () => {
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
  on:mousedown={(e) => {
    // Click outside the menu closes it
    if ($contextMenu.open && menuEl && !menuEl.contains(e.target as Node)) {
      closeContextMenu();
    }
  }}
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
          <RenderEmojis listitems={$resultItems} {activeIndex} />
        {:else if $resultType === "Markdown"}
          <RenderMarkdown listitems={$resultItems} {activeIndex} />
        {:else if $resultType === "Clipboard"}
          <RenderClipboard listitems={$resultItems} {activeIndex} />
        {:else if $resultType === "ColorPicker"}
          <RenderColorPicker />
        {:else if $resultType === "Home"}
          <RenderList listitems={$resultItems} {activeIndex} onContextMenu={handleContextMenu} />
        {:else if $resultType === "Math"}
          <RenderMath listitems={$resultItems} {activeIndex} />
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

  <!-- Context menu dropdown -->
  {#if $contextMenu.open && $contextMenu.item}
    <div
      class="ctx-menu"
      bind:this={menuEl}
      style="left: {$contextMenu.x}px; top: {$contextMenu.y}px;"
      transition:scale={{ duration: 120, start: 0.92, opacity: 0, easing: cubicOut }}
    >
      <div class="ctx-header">{$contextMenu.item.name}</div>
      {#each $contextMenu.item.actions as action, i}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_interactive_supports_focus -->
        <div
          class="ctx-item"
          class:ctx-active={i === $contextMenu.activeIndex}
          role="menuitem"
          on:click={() => {
            execute(action.id, $contextMenu.item!.name, $query);
            closeContextMenu();
          }}
          on:mouseenter={() => contextMenu.update(s => ({ ...s, activeIndex: i }))}
        >
          <span class="ctx-label">{action.name}</span>
          {#if i === 0}
            <span class="ctx-hint">default</span>
          {/if}
        </div>
      {/each}
    </div>
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
      Inter,
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

  /* ── Context menu ── */
  .ctx-menu {
    position: fixed;
    z-index: 500;
    min-width: 180px;
    max-width: 280px;
    background: rgba(28, 28, 30, 0.96);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 10px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.55), 0 2px 8px rgba(0, 0, 0, 0.3);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
    overflow: hidden;
    padding: 4px;
  }

  .ctx-header {
    padding: 6px 10px 4px;
    font-size: 0.7em;
    opacity: 0.45;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    user-select: none;
    color: var(--q-font-color, #ffffff);
  }

  .ctx-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 7px 10px;
    border-radius: 6px;
    cursor: pointer;
    transition: background 0.08s ease;
    user-select: none;
  }

  .ctx-item:hover,
  .ctx-item.ctx-active {
    background: var(--q-active-bg-color, rgba(255, 255, 255, 0.1));
  }

  .ctx-label {
    font-size: 0.9em;
    color: var(--q-font-color, #ffffff);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .ctx-hint {
    font-size: 0.7em;
    opacity: 0.38;
    white-space: nowrap;
    flex-shrink: 0;
    color: var(--q-font-color, #ffffff);
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
    background: rgba(38, 38, 40, 0.90);
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
