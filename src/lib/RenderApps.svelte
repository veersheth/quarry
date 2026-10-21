<script lang="ts">
  import { writable, type Writable } from "svelte/store";
  import type { ResultItem } from "../stores/search";
  import { mouseHasMoved } from "../stores/search";
  import { runItemAction } from "./keyHandler";
  import ItemIcon from "./ui/ItemIcon.svelte";

  export let listitems: ResultItem[] = [];
  export let activeIndex: Writable<number> = writable(0);
  export let indexOffset = 0;
  export let onContextMenu:
    | ((e: MouseEvent, item: ResultItem) => void)
    | undefined = undefined;

  $: {
    listitems;
    mouseHasMoved.set(false);
  }
</script>

<div class="app-grid">
  {#each listitems as item, index}
    {@const absoluteIndex = index + indexOffset}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      class="app-tile"
      class:active={absoluteIndex === $activeIndex}
      class:pinned={item.pinned}
      data-active={absoluteIndex === $activeIndex}
      on:mouseenter={() => {
        if ($mouseHasMoved) activeIndex.set(absoluteIndex);
      }}
      on:click={() => runItemAction(item)}
      on:contextmenu={(event) => {
        event.preventDefault();
        onContextMenu?.(event, item);
      }}
    >
      {#if item.icon}
        <ItemIcon icon={item.icon} name={item.name} size={40} />
      {:else}
        <span class="app-fallback">{item.name.trim().charAt(0).toUpperCase() || "?"}</span>
      {/if}
      <span class="app-name">{item.name}</span>
    </div>
  {/each}
</div>

<style>
  .app-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(104px, 1fr));
    gap: 10px;
    padding: 18px 22px 24px;
    overflow-y: auto;
    min-height: 0;
  }

  .app-tile {
    display: flex;
    min-width: 0;
    min-height: 94px;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 9px;
    padding: 12px 8px;
    border: 1px solid var(--q-border-subtle);
    border-radius: var(--q-item-border-radius);
    color: var(--q-text-secondary);
    text-align: center;
    transition: transform 100ms ease, border-color 100ms ease;
  }

  .app-tile.pinned { border-color: var(--q-pin-border); }

  .app-tile.active {
    background: var(--q-active-bg-color);
    border-color: var(--q-active-border-color);
    box-shadow: 0 0 16px 3px var(--q-glow);
    color: var(--q-font-color, #fff);
    transform: scale(1.04);
  }

  .app-tile.pinned.active { border-color: var(--q-pin-border-active); }

  .app-name {
    width: 100%;
    overflow: hidden;
    font-size: 0.86em;
    line-height: 1.25;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .app-fallback {
    display: grid;
    width: 40px;
    height: 40px;
    place-items: center;
    border-radius: 10px;
    background: var(--q-active-bg-color);
    color: var(--q-text-secondary);
    font-weight: 700;
  }
</style>
