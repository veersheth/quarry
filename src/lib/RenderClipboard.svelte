<script lang="ts">
  import { writable, type Writable } from "svelte/store";
  import type { ResultItem } from "../stores/search";
  import { runItemAction } from "./keyHandler";

  export let listitems: {
    name: string;
    action_id: string;
    description?: string;
    icon?: string;
  }[] = [];
  export let activeIndex: Writable<number> = writable(0);

  function handleClick(item: ResultItem) {
    runItemAction(item);
  }

  function truncate(str: string | undefined, maxLength: number): string {
    if (!str) return "";
    return str.length > maxLength ? str.slice(0, maxLength) + "…" : str;
  }

  function formatTimestamp(timestamp?: string | number): string {
    if (!timestamp) return "";
    const ts = typeof timestamp === "string" ? Number(timestamp) : timestamp;
    const date = new Date(ts * 1000);
    return `Copied at ${date.toLocaleString()}`;
  }
</script>

<div class="clipboard">
  <div class="result-list">
    {#each listitems as item, index}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="result-item"
        class:active={index === $activeIndex}
        on:mouseenter={() => activeIndex.set(index)}
        on:click={() => handleClick(item)}
      >
        <span class="item-name">{truncate(item.name, 16)}</span>
      </div>
    {/each}
  </div>
  <div class="info">
    <div class="preview">
      {listitems[$activeIndex]?.name}
    </div>
    <div class="data">
      {formatTimestamp(listitems[$activeIndex]?.description)}
    </div>
  </div>
</div>

<style>
  .clipboard {
    display: flex;
    flex-direction: row;
    height: 100%;
  }

  .result-list {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 12px 0 5px 0;
    height: 95%;
    overflow-y: auto;
  }

  .result-item {
    display: flex;
    align-items: center;
    width: auto;
    padding: 12px 18px;
    margin: 0 12px;
    border-radius: 12px;
    text-align: left;
    color: #e0e0e0;
    cursor: pointer;
    transition: transform 200ms ease;
  }

  .result-item.active {
    background-color: rgba(60, 60, 60, 0.7);
  }

  .info {
    flex: 3;
    display: flex;
    flex-direction: column;
    border-left: 1px solid rgba(80, 80, 80, 1);
  }

  .preview {
    flex: 9;
    overflow-y: auto;
    padding: 16px;
    font-family: "JetBrainsMono Nerd Font", "Courier New", Courier, monospace;
    white-space: pre-wrap;
    word-wrap: normal;
    color: rgba(255, 181, 188, 1);
  }

  .data {
    flex: 1;
    display: flex;
    justify-content: right;
    align-items: center;
    margin: 2px 32px;
    font-size: 0.9em;
    opacity: 0.3;
  }
</style>
