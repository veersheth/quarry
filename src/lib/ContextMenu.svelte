<script lang="ts">
  import { scale } from "svelte/transition";
  import { cubicOut, cubicIn } from "svelte/easing";
  import { execute } from "./searcher";
  import { query, contextMenu } from "../stores/search";
  import type { ResultItem } from "../stores/search";
  import { get } from "svelte/store";

  export let item: ResultItem;
  export let x: number;
  export let y: number;
  export let onClose: () => void;

  const MENU_W = 220;
  const GRID_THRESHOLD = 6;

  $: searchQ = $contextMenu.searchQuery ?? "";
  $: filteredActions = searchQ
    ? item.actions.filter((a) =>
        a.name.toLowerCase().includes(searchQ.toLowerCase()),
      )
    : item.actions;
  $: activeIdx = $contextMenu.activeIndex;
  $: isGrid = filteredActions.length > GRID_THRESHOLD;
  $: cols = isGrid ? 2 : 1;
  $: effectiveW = MENU_W * cols;

  let menuEl: HTMLDivElement;
  let listEl: HTMLDivElement;
  let menuActualH = 0;

  $: cx = Math.max(8, Math.min(x, window.innerWidth - effectiveW - 8));
  $: cy = Math.max(8, Math.min(y, window.innerHeight - (menuActualH || 300) - 8));

  function handleMousedown(e: MouseEvent) {
    if (menuEl && !menuEl.contains(e.target as Node)) {
      onClose();
    }
  }

  function selectAction(actionId: string) {
    execute(actionId, item.name, get(query));
    onClose();
  }
</script>

<svelte:window on:mousedown={handleMousedown} />

<div
  bind:this={menuEl}
  bind:clientHeight={menuActualH}
  class="ctx-menu"
  style="left: {cx}px; top: {cy}px"
  in:scale={{ duration: 160, start: 0.88, opacity: 0, easing: cubicOut }}
  out:scale={{ duration: 110, start: 0.94, opacity: 0, easing: cubicIn }}
  role="menu"
>
  {#if searchQ}
    <div class="ctx-search-display">
      <span class="ctx-search-text">{searchQ}</span><span class="ctx-cursor"
        >|</span
      >
    </div>
  {/if}

  <div class="ctx-list-wrap">
  <div
    class="ctx-list"
    class:grid-mode={isGrid}
    bind:this={listEl}
    style={isGrid ? `grid-template-rows: repeat(${Math.ceil(filteredActions.length / 2)}, auto)` : ""}
  >
  {#each filteredActions as action, i (action.id)}
    <!-- svelte-ignore a11y_interactive_supports_focus -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      class="ctx-item"
      class:active={i === activeIdx}
      role="menuitem"
      on:click={() => selectAction(action.id)}
      on:mouseenter={() =>
        contextMenu.update((s) => ({ ...s, activeIndex: i }))}
    >
      <span class="ctx-label">{action.name}</span>
    </div>
  {/each}

  {#if filteredActions.length === 0}
    <div class="ctx-empty">no match</div>
  {/if}
  </div>
  </div>

  {#if !searchQ}
    <div class="ctx-hint">type to filter</div>
  {/if}
</div>

<style>
  .ctx-menu {
    position: fixed;
    margin: 4px 20px;
    z-index: 500;
    min-width: 200px;
    max-width: 560px;
    font-family: var(--q-sans);
    background: var(--q-overlay);
    border: 2px solid var(--q-border-strong);
    border-radius: 24px;
    box-shadow: 0 0px 20px 5px rgba(0, 0, 0, 0.9);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(16px);
  }

  .ctx-search-display {
    display: flex;
    align-items: center;
    margin: 10px 10px 2px;
    padding: 6px 12px;
    background: var(--q-surface-subtle);
    border: 1px solid var(--q-border-medium);
    border-radius: 10px;
    font-size: 0.82em;
    color: var(--q-font-color, #fff);
    min-height: 28px;
  }

  .ctx-search-text {
    color: var(--q-font-color, #fff);
  }

  .ctx-cursor {
    opacity: 0.5;
    animation: blink 1s step-end infinite;
    color: var(--q-font-color, #fff);
    margin-left: 1px;
  }

  @keyframes blink {
    50% {
      opacity: 0;
    }
  }

  .ctx-empty {
    padding: 10px 18px 14px;
    font-size: 0.82em;
    opacity: 0.35;
    color: var(--q-font-color, #fff);
  }

  .ctx-hint {
    padding: 2px 0 10px;
    font-size: 0.7em;
    letter-spacing: 0.05em;
    opacity: 0.22;
    color: var(--q-font-color, #fff);
    text-align: center;
  }

  .ctx-list-wrap {
    position: relative;
  }

  .ctx-list {
    overflow: visible;
  }

  .ctx-list.grid-mode {
    display: grid;
    grid-template-columns: 1fr 1fr;
    grid-auto-flow: column;
  }

  .ctx-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    margin: 8px 6px;
    padding: 12px 14px;
    border-radius: 15px;
    user-select: none;
    color: var(--q-font-color, #fff);
  }

  .ctx-item:hover,
  .ctx-item.active {
    background: var(--q-surface-hover);
  }

  .ctx-label {
    font-size: 0.9em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--q-font-color, #fff);
  }
</style>
