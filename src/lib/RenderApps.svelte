<script lang="ts">
  import { writable, type Writable } from "svelte/store";
  import type { ResultItem } from "../stores/search";
  import GridRenderer from "./GridRenderer.svelte";
  import ItemIcon from "./ui/ItemIcon.svelte";

  export let listitems: ResultItem[] = [];
  export let activeIndex: Writable<number> = writable(0);
  export let indexOffset = 0;
  export let onContextMenu:
    | ((event: MouseEvent, item: ResultItem) => void)
    | undefined = undefined;
</script>

<div class="apps-grid">
  <GridRenderer
    {listitems}
    {activeIndex}
    {indexOffset}
    minCellWidth={104}
    gap={0}
    padding="12px 16px 16px"
    cellPadding="12px 8px"
    cellMinHeight="94px"
    {onContextMenu}
    let:item
  >
    {#if item.icon}
      <ItemIcon icon={item.icon} name={item.name} size={40} />
    {:else}
      <span class="app-fallback">{item.name.trim().charAt(0).toUpperCase() || "?"}</span>
    {/if}
    <span class="app-name">{item.name}</span>
  </GridRenderer>
</div>

<style>
  .apps-grid {
    height: 100%;
    min-height: 0;
  }

  .apps-grid :global(.grid-renderer) {
    height: 100%;
  }

  .apps-grid :global(.grid-cell) {
    flex-direction: column;
    gap: 12px;
  }

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
