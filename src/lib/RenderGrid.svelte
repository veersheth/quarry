<script lang="ts">
  import { writable, type Writable } from "svelte/store";
  import Icon from "./Icon.svelte";
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

<div class="result-grid">
  {#each listitems as item, index}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      class="grid-item"
      class:active={index === $activeIndex}
      on:mouseenter={() => activeIndex.set(index)}
      on:click={() => handleClick(item)}
    >
      <Icon icon={item.icon || ""} />
      <span class="item-name">{truncate(item.name, 20)}</span>
    </div>
  {/each}
</div>

<style>
  .result-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(90px, 1fr));
    gap: 12px;
    padding: 10px;
    margin: 2px;
  }

  .grid-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 28px 16px;
    border-radius: 12px;
    border: 1px solid rgba(60, 60, 60, 0.7);
    background-color: rgba(30, 30, 30, 0.5);
    cursor: pointer;
    text-align: center;
    transition: transform 100ms ease, border-radius 100ms ease;
  }

  .grid-item.active {
    background-color: rgba(60, 60, 60, 0.7);
    box-shadow: 0 0 20px 4px rgba(50, 50, 50, 1);
    border: 1px solid rgba(140, 140, 140, 0.9);
  }

  .grid-item:hover { transform: scale(1.08); border-radius: 20px; }

  .item-name {
    font-size: 0.9rem;
    color: #e0e0e0;
    word-break: break-word;
  }
</style>
