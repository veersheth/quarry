<script lang="ts">
  import { onMount, tick } from "svelte";
  import { execute } from "./searcher";
  import { query, contextMenu } from "../stores/search";
  import type { ResultItem } from "../stores/search";
  import { get } from "svelte/store";

  export let item: ResultItem;
  export let onClose: () => void;

  $: searchQ = $contextMenu.searchQuery ?? "";
  $: filteredActions = searchQ
    ? item.actions.filter((action) =>
        action.name.toLowerCase().includes(searchQ.toLowerCase()),
      )
    : item.actions;
  $: displayActions = filteredActions
    .map((action, index) => ({ action, index }))
    .reverse();
  $: activeIdx = $contextMenu.activeIndex;

  let menuEl: HTMLDivElement;
  let listEl: HTMLDivElement;
  let searchInput: HTMLInputElement;
  let readyToScroll = false;

  $: if (readyToScroll && listEl && activeIdx >= 0 && filteredActions.length > 0) {
    scrollActiveIntoView();
  }

  onMount(async () => {
    await tick();
    searchInput?.focus();
    listEl.scrollTop = listEl.scrollHeight;
    readyToScroll = true;
  });

  async function scrollActiveIntoView() {
    await tick();
    const activeItem = listEl?.querySelector<HTMLElement>(".ctx-item.active");
    if (!listEl || !activeItem) return;

    const padding = 8;
    const itemRect = activeItem.getBoundingClientRect();
    const listRect = listEl.getBoundingClientRect();
    const start = listEl.scrollTop;
    const visibleTop = start;
    const visibleBottom = start + listEl.clientHeight;
    const itemTop = itemRect.top - listRect.top + start;
    const itemBottom = itemTop + itemRect.height;
    let target = start;

    if (itemTop < visibleTop + padding) {
      target = itemTop - padding;
    } else if (itemBottom > visibleBottom - padding) {
      target = itemBottom - listEl.clientHeight + padding;
    }

    target = Math.max(0, Math.min(target, listEl.scrollHeight - listEl.clientHeight));
    listEl.scrollTop = target;
  }

  function handleMousedown(event: MouseEvent) {
    if (menuEl && !menuEl.contains(event.target as Node)) onClose();
  }

  function selectAction(actionId: string) {
    execute(actionId, item.name, get(query));
    onClose();
  }
</script>

<svelte:window on:mousedown={handleMousedown} />

<div bind:this={menuEl} class="ctx-menu" role="dialog" aria-label="Actions">
  <div bind:this={listEl} class="ctx-list" role="menu">
    {#each displayActions as entry (entry.action.id)}
      <!-- svelte-ignore a11y_interactive_supports_focus -->
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div
        class="ctx-item"
        class:active={entry.index === activeIdx}
        data-action-index={entry.index}
        role="menuitem"
        on:click={() => selectAction(entry.action.id)}
        on:mouseenter={() =>
          contextMenu.update((state) => ({ ...state, activeIndex: entry.index }))}
      >
        <span class="ctx-label">{entry.action.name}</span>
      </div>
    {/each}

    {#if filteredActions.length === 0}
      <div class="ctx-empty">No matching actions</div>
    {/if}
  </div>

  <label class="ctx-search">
    <input
      bind:this={searchInput}
      value={searchQ}
      placeholder="Filter actions"
      aria-label="Filter actions"
      spellcheck="false"
    />
    <kbd>esc</kbd>
  </label>
</div>

<style>
  .ctx-menu {
    position: fixed;
    right: 16px;
    bottom: 16px;
    z-index: 500;
    display: flex;
    flex-direction: column;
    width: min(340px, calc(100vw - 32px));
    max-height: min(480px, calc(100vh - 32px));
    padding: 10px;
    box-sizing: border-box;
    overflow: hidden;
    border: 1px solid var(--q-border-strong);
    border-radius: 18px;
    background: #1a1a1a;
    color: var(--q-font-color, #fff);
    font-family: var(--q-sans);
    box-shadow: 0 12px 36px rgba(0, 0, 0, 0.5);
  }

  .ctx-search {
    display: flex;
    flex: 0 0 auto;
    align-items: center;
    gap: 12px;
    min-height: 40px;
    padding: 0 11px;
    border: 1px solid var(--q-border-medium);
    border-radius: 11px;
    background: rgba(255, 255, 255, 0.045);
  }

  input {
    width: 100%;
    min-width: 0;
    padding: 0;
    border: 0;
    outline: none;
    background: transparent;
    color: var(--q-font-color, #fff);
    font: 0.9rem var(--q-sans);
  }

  input::placeholder {
    color: var(--q-text-dim);
  }

  kbd {
    flex: 0 0 auto;
    color: var(--q-text-dim);
    font: 0.68rem var(--q-mono);
  }

  .ctx-list {
    min-height: 0;
    margin-top: 8px;
    margin-bottom: 12px;
    overflow-y: auto;
  }

  .ctx-item {
    display: flex;
    align-items: center;
    min-height: 38px;
    margin: 2px 0;
    padding: 0 11px;
    border-radius: 9px;
    color: var(--q-font-color, #fff);
    user-select: none;
  }

  .ctx-item:hover,
  .ctx-item.active {
    background: rgba(255, 255, 255, 0.09);
  }

  .ctx-label {
    overflow: hidden;
    font-size: 0.88rem;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .ctx-empty {
    padding: 16px 11px;
    color: var(--q-text-dim);
    font-size: 0.84rem;
  }

</style>
