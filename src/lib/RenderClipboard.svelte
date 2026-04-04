<script lang="ts">
  import { writable, type Writable } from "svelte/store";
  import type { ResultItem } from "../stores/search";
  import { runItemAction } from "./keyHandler";

  export let listitems: {
    name: string;
    action_id: string;
    description?: string;
    icon?: string;
    thumbnail?: string;
  }[] = [];
  export let activeIndex: Writable<number> = writable(0);

  $: activeItem = listitems[$activeIndex];
  $: activeColor = activeItem?.thumbnail ? null : getValidColor(activeItem?.name);

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
    return `Copied ${date.toLocaleString()}`;
  }

  function getValidColor(str: string | undefined): string | null {
    if (!str) return null;
    const trimmed = str.trim();
    const standardRegex = /^(#([A-Fa-f0-9]{3,4}){1,2}|(rgb|hsl)a?\s*\(.*\))$/i;
    if (standardRegex.test(trimmed)) return trimmed;
    const nakedRgb = /^(\d{1,3}),\s*(\d{1,3}),\s*(\d{1,3})(,\s*[\d.]+)?$/;
    if (nakedRgb.test(trimmed)) return `rgb(${trimmed})`;
    const nakedHsl = /^(\d{1,3})°?,\s*(\d{1,3})%,\s*(\d{1,3})%(,\s*[\d.]+)?$/;
    if (nakedHsl.test(trimmed)) return `hsl(${trimmed.replace('°', '')})`;
    return null;
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
        class:image-item={!!item.thumbnail}
        on:mouseenter={() => activeIndex.set(index)}
        on:click={() => handleClick(item)}
      >
        {#if item.thumbnail}
          <img class="list-thumbnail" src={item.thumbnail} alt={item.name} />
        {:else}
          <span class="item-name">{truncate(item.name, 26)}</span>
          <div class="swatch-container">
            {#if getValidColor(item.name)}
              <div class="mini-swatch" style:background-color={getValidColor(item.name)}></div>
            {/if}
          </div>
        {/if}
      </div>
    {/each}
  </div>

  <div class="info-panel">
    {#if activeItem}
      <div class="preview-area">
        {#if activeItem.thumbnail}
          <img class="image-preview" src={activeItem.thumbnail} alt={activeItem.name} />
        {:else if activeColor}
          <div class="color-hero">
            <div class="checkerboard">
              <div class="main-swatch" style:background-color={activeColor}></div>
            </div>
            <code class="color-value">{activeItem.name}</code>
          </div>
        {:else}
          <div class="text-preview">
            {activeItem.name}
          </div>
        {/if}
      </div>

      <div class="metadata">
        <span class="timestamp">{formatTimestamp(activeItem.description)}</span>
      </div>
    {/if}
  </div>
</div>

<style>
  .clipboard {
    display: flex;
    height: 100%;
    color: #eee;
  }

  .result-list {
    flex: 0 0 218px;
    border-right: 1px solid #333;
    overflow-y: auto;
    padding: 8px;
  }

  .result-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 14px 14px;
    margin-bottom: 4px;
    border-radius: 12px;
    cursor: pointer;
    border: 2px solid rgba(255, 255, 255, 0);
  }

  .result-item.image-item {
    padding: 8px 20px;
    overflow: hidden;
  }

  .result-item.active {
    background: #2a2a2a;
    border: 2px solid rgba(255, 255, 255, 0.1);
  }

  .list-thumbnail {
    width: 100%;
    height: 72px;
    object-fit: cover;
    border-radius: 12px;
    display: block;
  }

  .item-name {
    flex: 1;
    font-size: 0.9rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-right: 10px;
  }

  .swatch-container {
    width: 18px;
    height: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .mini-swatch {
    width: 14px;
    height: 14px;
    border-radius: 12px;
  }

  .info-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .preview-area {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
    overflow: auto;
  }

  .image-preview {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    border-radius: 8px;
  }

  .color-hero {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 24px;
  }

  .checkerboard {
    width: 200px;
    height: 200px;
    border-radius: 111px;
    background-image: 
      linear-gradient(45deg, #222 25%, transparent 25%), 
      linear-gradient(-45deg, #222 25%, transparent 25%), 
      linear-gradient(45deg, transparent 75%, #222 75%), 
      linear-gradient(-45deg, transparent 75%, #222 75%);
    background-size: 20px 20px;
    background-position: 0 0, 0 10px, 10px -10px, -10px 0px;
    background-color: #111;
    overflow: hidden;
    border: 1px solid #333;
  }

  .main-swatch {
    width: 100%;
    height: 100%;
  }

  .color-value {
    font-family: 'JetBrainsMono Nerd Font', 'Fira Code', monospace;
    font-size: 1.1rem;
    background: #222;
    padding: 8px 18px;
    border-radius: 12px;
    color: #eee;
    border: 1px solid #333;
  }

  .text-preview {
    width: 100%;
    height: 100%;
    white-space: pre-wrap;
    word-break: break-all;
    font-family: 'JetBrains Mono', monospace;
    color: #ffb5bc;
    font-size: 0.95rem;
  }

  .metadata {
    padding: 12px 20px;
    border-top: 1px solid #333;
    text-align: right;
  }

  .timestamp {
    opacity: 0.4;
    font-size: 14px;
  }
</style>
