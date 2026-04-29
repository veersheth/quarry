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

  $: searchQ = $contextMenu.searchQuery ?? "";
  $: filteredActions = searchQ
    ? item.actions.filter((a) => a.name.toLowerCase().includes(searchQ.toLowerCase()))
    : item.actions;
  $: activeIdx = $contextMenu.activeIndex;
  $: menuH = (searchQ ? 52 : 24) + filteredActions.length * 52;

  $: cx = Math.max(8, Math.min(x, window.innerWidth - MENU_W - 8));
  $: cy = Math.max(8, Math.min(y, window.innerHeight - menuH - 8));

  let menuEl: HTMLDivElement;

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
  class="ctx-menu"
  style="left: {cx}px; top: {cy}px"
  in:scale={{ duration: 160, start: 0.88, opacity: 0, easing: cubicOut }}
  out:scale={{ duration: 110, start: 0.94, opacity: 0, easing: cubicIn }}
  role="menu"
>
  {#if searchQ}
    <div class="ctx-search-display">
      <span class="ctx-search-text">{searchQ}</span><span class="ctx-cursor">|</span>
    </div>
  {/if}

  {#each filteredActions as action, i (action.id)}
    <!-- svelte-ignore a11y_interactive_supports_focus -->
    <div
      class="ctx-item"
      class:active={i === activeIdx}
      role="menuitem"
      on:click={() => selectAction(action.id)}
      on:mouseenter={() => contextMenu.update(s => ({ ...s, activeIndex: i }))}
    >
      <span class="ctx-label">{action.name}</span>
      {#if action.id === item.actions[0].id}
        <span class="ctx-badge">default</span>
      {/if}
    </div>
  {/each}

  {#if filteredActions.length === 0}
    <div class="ctx-empty">no match</div>
  {/if}

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
    max-width: 280px;
    font-family: "Inter", "Segoe UI", "Adwaita Sans", "Noto Color Emoji", sans-serif;
    background: rgba(25, 25, 25, 0.80);
    border: 2px solid rgba(255, 255, 255, 0.15);
    border-radius: 24px;
    box-shadow: 0 0px 10px rgba(0, 0, 0, 0.4);
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(8px);
    overflow: hidden;
  }

  .ctx-search-display {
    display: flex;
    align-items: center;
    margin: 10px 10px 2px;
    padding: 6px 12px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.14);
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
    50% { opacity: 0; }
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

  .ctx-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    margin: 8px 6px;
    padding: 12px 14px;
    border-radius: 15px;
    cursor: pointer;
    user-select: none;
    color: var(--q-font-color, #fff);
  }

  .ctx-item:hover,
  .ctx-item.active {
    background: rgba(255, 255, 255, 0.20);
  }

  .ctx-label {
    font-size: 0.9em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--q-font-color, #fff);
  }

  .ctx-badge {
    font-size: 0.67em;
    letter-spacing: 0.05em;
    opacity: 0.28;
    white-space: nowrap;
    flex-shrink: 0;
    color: var(--q-font-color, #fff);
  }
</style>
