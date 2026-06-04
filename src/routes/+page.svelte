<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
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
    searcherName,
    mouseHasMoved,
    modalStore,
    openModal,
    closeModal,
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
  import RenderScreenshots from "$lib/RenderScreenshots.svelte";
  import Modal from "$lib/Modal.svelte";

  function searcherHue(name: string): number {
    let h = 0;
    for (let i = 0; i < name.length; i++) h = (h * 31 + name.charCodeAt(i)) & 0xffff;
    return h % 360;
  }

  let searchInput: HTMLInputElement;
  let appWindow: ReturnType<typeof getCurrentWindow>;
  let isLoading = false;
  let aiPrefix = "ai ";
  let resultsEl: HTMLDivElement;
  let searchRaf: number;
  let uiScale = 1.0;

  // Rofi mode: bypass backend search, filter items locally
  let rofiMode = false;
  let rofiResponseSocket = "";
  let rofiAllItems: import("../stores/search").ResultItem[] = [];

  function filterRofiItems(q: string) {
    if (!q) {
      resultItems.set(rofiAllItems);
      return;
    }
    const lower = q.toLowerCase();
    resultItems.set(
      rofiAllItems.filter((item) => item.name.toLowerCase().includes(lower)),
    );
  }

  async function cancelRofi() {
    if (!rofiMode) return;
    rofiMode = false;
    rofiAllItems = [];
    try {
      await invoke("cancel_rofi", { responseSocket: rofiResponseSocket });
    } catch (_) {}
    rofiResponseSocket = "";
    query.set("");
    resultItems.set([]);
  }

  function handleContextMenu(
    event: MouseEvent,
    item: import("../stores/search").ResultItem,
  ) {
    if (item.actions.length <= 1) return;
    event.preventDefault();
    openContextMenu(item, event.clientX, event.clientY);
  }

  function handleOpenAtActive() {
    const items = $resultItems;
    const idx = $activeIndex;
    const item = items[idx];
    if (!item || item.actions.length <= 1) return;

    const activeRow = resultsEl?.querySelector(
      "[data-active='true']",
    ) as HTMLElement | null;
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
    font_family: string;
    monospace_font_family: string;
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
    root.setProperty("--q-font-family", t.font_family);
    root.setProperty("--q-monospace-font-family", t.monospace_font_family);
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
    forceRepaint();
    searchInput?.select();
  }

  onMount(async () => {
    appWindow = getCurrentWindow();
    await refresh();
    invoke<string>("get_ai_prefix")
      .then((p) => {
        aiPrefix = p;
      })
      .catch(() => {});

    // use query from the commandline
    invoke<string | null>("take_pending_query").then((q) => {
      if (q) query.set(q);
    }).catch(() => {});
    const unlisten = appWindow.onFocusChanged(({ payload: focused }) => {
      if (focused) {
        mouseHasMoved.set(false);
        refresh();
      } else if (rofiMode) {
        cancelRofi();
      }
    });

    window.addEventListener("open-context-menu-at-active", handleOpenAtActive);

    const blockZoom = (e: WheelEvent) => {
      if (e.ctrlKey) e.preventDefault();
    };
    window.addEventListener("wheel", blockZoom, { passive: false });

    const unlistenModal = await listen<{ body: string; buttons?: { label: string; kind?: string; shell?: string }[] }>(
      "quarry-modal",
      (event) => {
        openModal(event.payload.body, event.payload.buttons);
      },
    );

    const unlistenRofiSocket = await listen<string>(
      "quarry-rofi-socket",
      (event) => {
        rofiResponseSocket = event.payload;
      },
    );

    const unlistenRofi = await listen<import("../stores/search").SearchResult>(
      "quarry-rofi",
      (event) => {
        const res = event.payload;
        rofiAllItems = res.results;
        rofiMode = true;
        resultItems.set(res.results);
        resultType.set(res.result_type);
        query.set("");
        activeIndex.set(0);
      },
    );

    // Fast partial results from the default searcher (apps/system/shortcuts/bookmarks)
    // arrive before the file search completes — apply immediately if still relevant.
    const unlistenFast = await listen<{
      query: string;
      results: import("../stores/search").ResultItem[];
    }>("quarry-fast", ({ payload }) => {
      if (!rofiMode && payload.query === $query.trim()) {
        resultItems.set(payload.results);
        resultType.set("List");
        activeIndex.set(0);
      }
    });

    // Pre-fill query from a subsequent `quarry <query>` invocation.
    const unlistenSetQuery = await listen<string>("quarry-set-query", ({ payload }) => {
      query.set(payload);
      searchInput?.focus();
    });

    return () => {
      unlisten.then((fn) => fn());
      unlistenModal();
      unlistenRofiSocket();
      unlistenRofi();
      unlistenFast();
      unlistenSetQuery();
      window.removeEventListener("open-context-menu-at-active", handleOpenAtActive);
      window.removeEventListener("wheel", blockZoom);
    };
  });

  $: if ($resultItems) {
    if (resultsEl) resultsEl.scrollTop = 0;
  }

  $: if ($query !== undefined) {
    if (rofiMode) {
      filterRofiItems($query);
      activeIndex.set(0);
    } else {
      isLoading = true;
      cancelAnimationFrame(searchRaf);
      searchRaf = requestAnimationFrame(() => {
        search($query)
          .then((res) => {
            if (res === null) return;
            resultItems.set(res.results);
            resultType.set(res.result_type);
            searcherName.set(res.searcher ?? "");
            activeIndex.set(0);
          })
          .catch((err) => {
            console.error("Search error:", err);
            resultItems.set([]);
          })
          .finally(() => {
            isLoading = false;
          });
      });
    }
  }
</script>

<svelte:window
  on:keydown={(e) => {
    if (e.ctrlKey && (e.key === "-" || e.key === "=" || e.key === "+" || e.key === "0")) {
      e.preventDefault();
      if (e.key === "-") uiScale = Math.max(0.6, parseFloat((uiScale - 0.05).toFixed(2)));
      else if (e.key === "=" || e.key === "+") uiScale = Math.min(2.0, parseFloat((uiScale + 0.05).toFixed(2)));
      else if (e.key === "0") uiScale = 1.0;
      return;
    }
    handleKeydown(e, searchInput, activeIndex, resultItems, appWindow, aiPrefix);
  }}
  on:pointermove={(e) => { if (e.movementX !== 0 || e.movementY !== 0) mouseHasMoved.set(true); }}
/>

<main class="container" style="zoom: {uiScale}">
  <div class="panel">
    <div class="search-bar" class:searcher-active={!!$searcherName} style={$searcherName ? `--searcher-hue: ${searcherHue($searcherName)}` : ""}>
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
      {#if $searcherName}
        <span class="searcher-badge">{$searcherName}</span>
      {/if}
    </div>
    <div
      class="results"
      class:loading-overlay={isLoading}
      bind:this={resultsEl}
    >
      <div class="results-content" class:dimmed={isLoading}>
        {#if $resultType === "List"}
          <RenderList
            listitems={$resultItems}
            {activeIndex}
            onContextMenu={handleContextMenu}
          />
        {:else if $resultType === "Grid"}
          <RenderEmojis
            listitems={$resultItems}
            {activeIndex}
            onContextMenu={handleContextMenu}
          />
        {:else if $resultType === "Markdown"}
          <RenderMarkdown listitems={$resultItems} {activeIndex} />
        {:else if $resultType === "Clipboard"}
          <RenderClipboard
            listitems={$resultItems}
            {activeIndex}
            onContextMenu={handleContextMenu}
          />
        {:else if $resultType === "ColorPicker"}
          <RenderColorPicker initialColor={$resultItems[0]?.name ?? ""} />
        {:else if $resultType === "Home"}
          <RenderList
            listitems={$resultItems}
            {activeIndex}
            onContextMenu={handleContextMenu}
          />
        {:else if $resultType === "Math"}
          <RenderMath
            listitems={$resultItems}
            {activeIndex}
            onContextMenu={handleContextMenu}
          />
        {:else if $resultType === "Camera"}
          <RenderCamera />
        {:else if $resultType === "Ai"}
          <RenderAiChat />
        {:else if $resultType === "Screenshots"}
          <RenderScreenshots
            listitems={$resultItems}
            {activeIndex}
            onContextMenu={handleContextMenu}
          />
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

  {#if $modalStore.open}
    <Modal />
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
  :global(*) {
    -webkit-user-select: none !important;
    user-select: none !important;
    touch-action: pan-x pan-y;
  }

  :global(input),
  :global(textarea) {
    -webkit-user-select: text !important;
    user-select: text !important;
  }

  :global(:root) {
    --q-sans: var(--q-font-family, "Inter", "Segoe UI", "Adwaita Sans", "Noto Color Emoji", sans-serif);
    --q-mono: var(--q-monospace-font-family, "JetBrainsMono Nerd Font", "Fira Code", "Cascadia Mono", monospace);

    --q-surface: rgba(5, 5, 5, 0.8);
    --q-surface-subtle: rgba(255, 255, 255, 0.08);
    --q-surface-hover: rgba(255, 255, 255, 0.2);
    --q-surface-dark: #222;
    --q-overlay: rgba(30, 30, 30, 0.8);
    --q-code-bg: #181818;

    --q-divider: rgba(80, 80, 80, 0.7);
    --q-divider-dark: #2e2e2e;
    --q-border-subtle: rgba(60, 60, 60, 0.3);
    --q-border-medium: rgba(255, 255, 255, 0.15);
    --q-border-strong: rgba(255, 255, 255, 0.25);
    --q-border-dark: #333;

    --q-text-secondary: color-mix(in srgb, var(--q-font-color, #ffffff) 70%, transparent);
    --q-text-muted: color-mix(in srgb, var(--q-font-color, #ffffff) 40%, transparent);
    --q-text-dim: color-mix(in srgb, var(--q-font-color, #ffffff) 25%, transparent);
    --q-text-dim-active: color-mix(in srgb, var(--q-font-color, #ffffff) 55%, transparent);
    --q-text-placeholder: color-mix(in srgb, var(--q-font-color, #ffffff) 15%, transparent);
    --q-text-empty: color-mix(in srgb, var(--q-font-color, #ffffff) 20%, transparent);

    --q-thumb-bg: #111;
    --q-pin-border: rgba(200, 220, 255, 0.25);
    --q-pin-border-active: rgba(210, 230, 255, 0.55);
    --q-glow: rgba(50, 50, 50, 1);
  }

  .container {
    display: flex;
    flex: 1;
    height: calc(100vh - 2px);
    flex-direction: column;
    margin: 0;
    padding: 0;
    box-sizing: border-box;
    background-color: var(--q-bg-color, rgba(10, 10, 10, 1));
    opacity: var(--q-bg-opacity, 1);
    overflow: hidden;
    color: var(--q-font-color, #ffffff);
    position: relative;
    border-style: inset;
    border: var(--q-border-thickness, 1px) solid var(--q-border-color, rgba(255,255,255,0.35));
    border-radius: var(--q-border-radius, 18px);
  }

  .container * {
    z-index: 0;
    color: var(--q-font-color, #ffffff);
    font-size: var(--q-font-size);
    font-family: var(--q-sans);
    padding: 0;
  }

  .panel {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
  }

  .search-bar {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 20px;
    height: 56px;
    flex-shrink: 0;
    margin-bottom: 8px;
    border-bottom: 1px solid var(--q-divider);
    position: relative;
    overflow: hidden;
  }

  .search-bar::before {
    content: "";
    position: absolute;
    inset: 0;
    background: linear-gradient(
      90deg,
      hsla(var(--searcher-hue, 220), 75%, 65%, 0.12) 0%,
      transparent 95%
    );

    opacity: 0;
    pointer-events: none;
    transition: opacity 0.4s ease;
  }

  .search-bar.searcher-active::before {
    opacity: 1;
  }

  .searcher-badge {
    font-size: 0.72rem;
    font-family: var(--q-mono);
    padding: 3px 9px;
    border-radius: 6px;
    border: 1px solid var(--q-border-medium);
    border-color: hsla(var(--searcher-hue, 220), 75%, 65%, 0.42);
    color: hsla(var(--searcher-hue, 220), 75%, 72%, 0.92);
    white-space: nowrap;
    flex-shrink: 0;
    letter-spacing: 0.03em;
  }

  .search {
    flex: 1;
    min-width: 0;
    border: none;
    outline: none;
    background: none;
    height: 100%;
    padding: 0;
    transition: opacity 0.15s ease;
  }

  .search.loading {
    opacity: 0.7;
  }

  .results {
    margin: 0;
    padding: 0;
    border-top: none;
    background-size: 100% 1px;
    background-repeat: no-repeat;
    background-position: top;
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
    color: var(--q-text-secondary);
    background: rgba(5, 5, 5, 0.2);
    border: 2px solid var(--q-border-medium);
    box-shadow: 0 0px 10px rgba(0, 0, 0, 0.4);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
  }

  .toast-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .toast-dot.success {
    background: #4ade80;
  }

  .toast-dot.error {
    background: #f87171;
  }

  .toast-dot.info {
    background: #60a5fa;
  }

  .toast.error {
    border-color: rgba(248, 113, 113, 0.2);
  }

  .toast.info {
    border-color: rgba(96, 165, 250, 0.2);
  }
</style>
