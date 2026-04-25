<script lang="ts">
  import { scale } from "svelte/transition";
  import { cubicOut, cubicIn } from "svelte/easing";
  import { execute } from "./searcher";
  import { query } from "../stores/search";
  import type { ResultItem } from "../stores/search";
  import { get } from "svelte/store";

  export let item: ResultItem;
  export let x: number;
  export let y: number;
  export let onClose: () => void;

  const MENU_W = 220;
  $: menuH = 46 + item.actions.length * 36;

  $: cx = Math.max(8, Math.min(x, window.innerWidth - MENU_W - 8));
  $: cy = Math.max(8, Math.min(y, window.innerHeight - menuH - 8));

  let activeIdx = 0;
  let menuEl: HTMLDivElement;

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      onClose();
      return;
    }
    if (e.key === "ArrowDown" || (e.key === "n" && e.ctrlKey)) {
      e.preventDefault();
      e.stopPropagation();
      activeIdx = (activeIdx + 1) % item.actions.length;
    }
    if (e.key === "ArrowUp" || (e.key === "p" && e.ctrlKey)) {
      e.preventDefault();
      e.stopPropagation();
      activeIdx = (activeIdx - 1 + item.actions.length) % item.actions.length;
    }
    if (e.key === "Enter") {
      e.preventDefault();
      e.stopPropagation();
      selectAction(item.actions[activeIdx].id);
    }
  }

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

<svelte:window on:keydown={handleKeydown} on:mousedown={handleMousedown} />

<div
  bind:this={menuEl}
  class="ctx-menu"
  style="left: {cx}px; top: {cy}px"
  in:scale={{ duration: 160, start: 0.88, opacity: 0, easing: cubicOut }}
  out:scale={{ duration: 110, start: 0.94, opacity: 0, easing: cubicIn }}
  role="menu"
>
  {#each item.actions as action, i}
    <!-- svelte-ignore a11y_interactive_supports_focus -->
    <div
      class="ctx-item"
      class:active={i === activeIdx}
      role="menuitem"
      on:click={() => selectAction(action.id)}
      on:mouseenter={() => (activeIdx = i)}
    >
      <span class="ctx-label">{action.name}</span>
      {#if i === 0}
        <span class="ctx-badge">default</span>
      {/if}
    </div>
  {/each}
</div>

<style>
  .ctx-menu {
    position: fixed;
    margin: 4px 20px;
    z-index: 500;
    min-width: 200px;
    max-width: 280px;
    font-family: "Inter", "Segoe UI", "Adwaita Sans", "Noto Color Emoji", sans-serif;
    background: rgba(25, 25, 25, 0.00);
    border: 2px solid rgba(255, 255, 255, 0.15);
    border-radius: 24px;
    box-shadow:
      0 0px 10px rgba(0, 0, 0, 0.4);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    overflow: hidden;
  }

  .ctx-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    margin: 8px 6px;
    padding: 12px 14px;
    border-radius: 12px;
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
