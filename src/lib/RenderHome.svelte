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
      {#if index < 4}
        <div class="shortcut">Alt + {index + 1}</div>
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
    border: none;
    border-radius: 12px;
    background: none;
    text-align: left;
    color: #e0e0e0;
    cursor: pointer;
    border: 1px solid rgba(0, 0, 0, 0);
    transition: transform 200ms ease;
  }

  .result-item.active {
    background-color: rgba(255, 255, 255, 0.1);
  }

  img.item-icon {
    width: 20px;
    height: 20px;
    margin-right: 0.7rem;
    display: inline-block;

    object-fit: contain;
    object-position: center;
  }

  .item-name {
    margin-left: 0;
    margin-right: 0.7rem;
  }

  .item-desc {
    opacity: 0.4;
    font-size: 16px;
  }

  .shortcut {
    margin-left: auto;
    opacity: 0.4;
    font-size: 14px;
    font-family: monospace;
  }
</style>
