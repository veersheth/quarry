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
    padding: 12px 18px;
    margin: 0 12px;
    border-radius: var(--q-item-border-radius);
    background: none;
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
    margin-right: 0.7rem;
    flex-shrink: 0;
    object-fit: contain;
    object-position: center;
  }

  .item-text {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 0.8rem;
    overflow: hidden;
  }

  .item-name {
    <!-- font-size: 15px; -->
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex-shrink: 0;
    max-width: 40%;
  }

  .item-desc {
    opacity: 0.4;
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
  }
</style>
