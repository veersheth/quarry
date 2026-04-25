<script lang="ts">
  import { writable, type Writable } from "svelte/store";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import type { ResultItem } from "../stores/search";
  import { runItemAction } from "./keyHandler";

  function iconSrc(icon: string): string {
    return icon.startsWith("/") ? convertFileSrc(icon) : icon;
  }

  function handleClick(item: ResultItem) {
    runItemAction(item);
  }

  export let listitems: {
    name: string;
    actions: { id: string; name: string }[];
    description?: string;
    icon?: string;
  }[] = [];
  export let activeIndex: Writable<number> = writable(0);
  export let onContextMenu: ((e: MouseEvent, item: (typeof listitems)[number]) => void) | undefined = undefined;
</script>

<div class="result-list">
  {#each listitems as item, index}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      class="result-item"
      class:active={index === $activeIndex}
      class:has-desc={!!item.description}
      data-active={index === $activeIndex}
      on:mouseenter={() => activeIndex.set(index)}
      on:click={() => handleClick(item)}
      on:contextmenu={(e) => { e.preventDefault(); onContextMenu?.(e, item); }}
    >
      {#if item.icon}
        <img
          class="item-icon"
          src={iconSrc(item.icon)}
          alt=""
        />
      {/if}
      <div class="item-text">
        <span class="item-name">{item.name}</span>
        {#if item.description}
          <span class="item-desc">{item.description}</span>
        {/if}
      </div>
    </div>
  {/each}
</div>

<style>
  .result-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 15px 0 5px;
  }

  .result-item {
    display: flex;
    align-items: center;
    width: auto;
    padding: 12px 10px;
    margin: 0 12px;
    border-radius: var(--q-item-border-radius);
    background: none;
    background: rgba(3, 3, 3, 0.8);
    text-align: left;
    color: rgba(255, 255, 255, 0.7);
    cursor: pointer;
    border: 2px solid transparent;
  }

  .result-item.active {
    background-color: var(--q-active-bg-color);
    border: 2px solid var(--q-active-border-color);
  }

  .result-item.active .item-icon {
    filter: drop-shadow(0 0 4px rgba(255, 255, 255, 0.3));
  }

  img.item-icon {
    width: 20px;
    height: 20px;
    margin-right: 14px;
    flex-shrink: 0;
    object-fit: contain;
    object-position: center;
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
    font-size: 1rem;
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
