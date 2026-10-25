<script lang="ts">
  import { writable, type Writable } from "svelte/store";
  import type { ResultItem } from "../stores/search";
  import GridRenderer from "./GridRenderer.svelte";

  export let listitems: ResultItem[] = [];
  export let activeIndex: Writable<number> = writable(0);
  export let onContextMenu:
    | ((event: MouseEvent, item: ResultItem) => void)
    | undefined = undefined;

  $: featured = listitems.filter((item) => item.group === "featured");
  $: all = listitems.filter((item) => item.group !== "featured");
  $: allOffset = featured.length;
</script>

<div class="emoji-layout">
  {#if featured.length > 0}
    <GridRenderer
      listitems={featured}
      {activeIndex}
      layout="strip"
      minCellWidth={48}
      gap={0}
      padding="0"
      cellPadding="0"
      cellMinHeight="48px"
      {onContextMenu}
      let:item
    >
      <span class="emoji featured-emoji">{item.name}</span>
    </GridRenderer>
    <div class="section-divider"></div>
  {/if}

  <GridRenderer
    listitems={all}
    {activeIndex}
    indexOffset={allOffset}
    minCellWidth={76}
    gap={0}
    padding="0"
    cellPadding="0"
    {onContextMenu}
    let:item
  >
    <span class="emoji">{item.name}</span>
  </GridRenderer>
</div>

<style>
  .emoji-layout {
    display: flex;
    height: 100%;
    min-height: 0;
    flex-direction: column;
    box-sizing: border-box;
    padding: 12px 16px 16px;
  }

  .emoji-layout :global(.grid-cell) {
    flex-direction: column;
  }

  .emoji-layout :global(.grid-renderer:not(.strip) .grid-cell) {
    min-height: 76px;
    padding: 0;
  }

  .emoji-layout :global(.grid-renderer:not(.strip)) {
    flex: 1;
    grid-auto-rows: 76px;
  }

  .emoji-layout :global(.grid-renderer.strip .grid-cell) {
    flex: 0 0 48px;
    width: 48px;
  }

  .section-divider {
    display: flex;
    flex-shrink: 0;
    align-items: center;
    gap: 8px;
    margin: 10px;
    padding: 0 10px 6px;
  }

  .emoji {
    font-size: 2em;
    line-height: 1;
  }

  .featured-emoji {
    margin-bottom: 0;
    font-size: 1.5em;
  }
</style>
