<script lang="ts">
  import { writable, type Writable } from "svelte/store";
  import type { ResultItem } from "../stores/search";
  import { runItemAction } from "./keyHandler";

  function handleClick(item: ResultItem) {
    runItemAction(item);
  }

  export let listitems: {
    name: string;
    action_id: string;
    description?: string;
    icon?: string;
  }[] = [];

  export let activeIndex: Writable<number> = writable(0);

  function truncate(str: string | undefined, maxLength: number): string {
    if (!str) return "";
    return str.length > maxLength ? str.slice(0, maxLength) + "…" : str;
  }

  function getBadge(action_id: string): string {
    if (action_id.includes("search_google")) return "google";
    if (action_id.includes("search_youtube")) return "youtube";
    if (action_id.includes("shell")) return "shell";
    if (action_id.includes("url") || action_id.includes("http")) return "url";
    return "app";
  }
</script>

<div class="result-list">
  {#each listitems as item, index}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      class="result-item"
      class:active={index === $activeIndex}
      on:mouseenter={() => activeIndex.set(index)}
      on:click={() => handleClick(item)}
    >
      {#if item.icon}
        <img
          class="item-icon"
          src={item.icon}
          alt=""
          on:error={(e) => {
            e.currentTarget.style.display = "none";
          }}
        />
      {/if}
      <span class="item-name">{truncate(item.name, 80)}</span>
      {#if item.description}
        <span class="item-desc">{truncate(item.description, 70)}</span>
      {/if}
      <span class="item-badge">{getBadge(item.action_id)}</span>
      {#if index === $activeIndex}
        <span class="item-enter">↵</span>
      {/if}
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
    padding: 12px 18px;
    margin: 0 12px;
    border-radius: 10px;
    background: none;
    text-align: left;
    color: #e0e0e0;
    cursor: pointer;
    border: 2px solid transparent;
  }

  .result-item.active {
    background-color: rgba(106, 147, 154, 0.1);
    border: 2px solid rgba(106, 147, 154, 0.5);
  }

  .result-item.active .item-name {
    color: rgba(156, 197, 204, 1);
  }

  .result-item.active .item-desc {
    color: rgba(156, 197, 204, 0.5);
  }

  img.item-icon {
    width: 20px;
    height: 20px;
    margin-right: 0.7rem;
    flex-shrink: 0;
    object-fit: contain;
    object-position: center;
  }

  .item-name {
    margin-right: 0.8rem;
  }

  .item-desc {
    opacity: 0.4;
    font-size: 14px;
  }

  .item-badge {
    margin-left: auto;
    font-size: 10px;
    color: rgba(106, 147, 154, 0.5);
    border: 1px solid rgba(106, 147, 154, 0.2);
    border-radius: 4px;
    padding: 1px 6px;
    flex-shrink: 0;
  }

  .item-enter {
    font-size: 11px;
    color: rgba(156, 197, 204, 0.6);
    margin-left: 8px;
    flex-shrink: 0;
  }
</style>
