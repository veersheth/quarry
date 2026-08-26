<script lang="ts">
  import { writable, type Writable } from "svelte/store";
  import type { ResultItem } from "../stores/search";
  import { mouseHasMoved } from "../stores/search";
  import { runItemAction } from "./keyHandler";
  import ItemIcon from "./ui/ItemIcon.svelte";

  export let listitems: ResultItem[] = [];
  export let activeIndex: Writable<number> = writable(0);
  export let indexOffset: number = 0;
  export let onContextMenu:
    | ((e: MouseEvent, item: ResultItem) => void)
    | undefined = undefined;

  $: {
    listitems;
    mouseHasMoved.set(false);
  }
</script>

<div class="result-list">
  {#each listitems as item, index}
    {#if item.group === "header"}
      <div class="section-header">{item.name}</div>
    {:else}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div
        class="result-item"
        class:active={index === $activeIndex - indexOffset}
        class:has-desc={!!item.description}
        class:pinned={item.pinned}
        data-active={index === $activeIndex - indexOffset}
        on:mouseenter={() => {
          if ($mouseHasMoved) activeIndex.set(index + indexOffset);
        }}
        on:click={() => runItemAction(item)}
        on:contextmenu={(e) => {
          e.preventDefault();
          onContextMenu?.(e, item);
        }}
        on:dragstart|preventDefault
      >
        {#if item.icon}
          <ItemIcon icon={item.icon} name={item.name} draggable_path={item.draggable_path} />
        {/if}
        <div class="item-text">
          <span class="item-name">{item.name}</span>
          {#if item.description}
            <span class="item-desc">{item.description}</span>
          {/if}
        </div>
      </div>
    {/if}
  {/each}
</div>

<style>
  .result-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 5px 0;
  }

  .section-header {
    padding: 10px 22px 3px;
    font-size: 0.72em;
    font-family: var(--q-mono);
    color: var(--q-text-dim);
    user-select: none;
    pointer-events: none;
  }

  .result-item {
    font-size: 1em;
    display: flex;
    align-items: center;
    gap: 14px;
    width: auto;
    padding: 12px 10px;
    margin: 0 12px;
    border-radius: var(--q-item-border-radius);
    background: none;
    background: var(--q-surface);
    text-align: left;
    color: var(--q-text-secondary);
    border: 2px solid transparent;
  }

  .result-item.pinned {
    border-color: var(--q-pin-border);
  }

  .result-item.active {
    background-color: var(--q-active-bg-color);
    border: 2px solid var(--q-active-border-color);
  }

  .result-item.pinned.active {
    border-color: var(--q-pin-border-active);
  }

  .item-text {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: baseline;
    gap: 0.8rem;
    overflow: hidden;
  }

  .item-name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex-shrink: 1;
    min-width: 0;
  }

  /* Only cap name width when a description is also present */
  .has-desc .item-name {
    flex-shrink: 0;
    max-width: 50%;
  }

  .item-desc {
    opacity: 0.4;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    min-width: 0;
  }

</style>
