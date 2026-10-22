<script lang="ts">
  import { writable, type Writable } from "svelte/store";
  import type { ResultItem } from "../stores/search";
  import { mouseHasMoved } from "../stores/search";
  import { runItemAction } from "./keyHandler";

  export let listitems: ResultItem[] = [];
  export let activeIndex: Writable<number> = writable(0);
  export let indexOffset = 0;
  export let layout: "grid" | "strip" = "grid";
  export let minCellWidth = 90;
  export let gap = 12;
  export let padding = "24px";
  export let cellPadding = "28px 16px";
  export let cellMinHeight = "0px";
  export let onContextMenu:
    | ((event: MouseEvent, item: ResultItem) => void)
    | undefined = undefined;

  $: {
    listitems;
    mouseHasMoved.set(false);
  }
</script>

<div
  class="grid-renderer"
  class:strip={layout === "strip"}
  style={`--grid-min-width:${minCellWidth}px;--grid-gap:${gap}px;--grid-padding:${padding};--grid-cell-padding:${cellPadding};--grid-cell-min-height:${cellMinHeight}`}
>
  {#each listitems as item, index}
    {@const absoluteIndex = index + indexOffset}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      class="grid-cell"
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
      <slot {item} index={absoluteIndex} />
    </div>
  {/each}
</div>

<style>
  .grid-renderer {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(var(--grid-min-width), 1fr));
    align-content: start;
    gap: var(--grid-gap);
    min-height: 0;
    overflow-y: auto;
    padding: var(--grid-padding);
  }

  .grid-renderer.strip {
    display: flex;
    flex-wrap: wrap;
    overflow: visible;
  }

  .grid-cell {
    display: flex;
    min-width: 0;
    min-height: var(--grid-cell-min-height);
    align-items: center;
    justify-content: center;
    padding: var(--grid-cell-padding);
    border: 1px solid var(--q-border-subtle);
    border-radius: var(--q-item-border-radius);
    color: var(--q-text-secondary);
    text-align: center;
  }

  .grid-cell.pinned {
    border-color: var(--q-pin-border);
  }

  .grid-cell.active {
    background: var(--q-active-bg-color);
    border-color: var(--q-active-border-color);
    box-shadow: 0 0 16px 3px var(--q-glow);
    color: var(--q-font-color, #fff);
  }

  .grid-cell.pinned.active {
    border-color: var(--q-pin-border-active);
  }

  .strip .grid-cell {
    flex: 0 0 auto;
  }
</style>
