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

  function handleClick(item: typeof listitems[0]) {
    runItemAction(item as unknown as ResultItem);
  }

  function truncate(str: string | undefined, maxLength: number): string {
    if (!str) return "";
    return str.length > maxLength ? str.slice(0, maxLength) + "…" : str;
  }

  function formatTimestamp(timestamp?: string | number): string {
    if (!timestamp) return "";
    const ts = typeof timestamp === "string" ? Number(timestamp) : timestamp;
    if (isNaN(ts)) return String(timestamp);
    const now = Date.now() / 1000;
    const age = now - ts;
    if (age < 60) return "just now";
    if (age < 3600) return `${Math.floor(age / 60)} min ago`;
    if (age < 86400) return `${Math.floor(age / 3600)} hr ago`;
    return `${Math.floor(age / 86400)} days ago`;
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

  function itemType(item: typeof listitems[0]): "image" | "color" | "text" {
    if (item.thumbnail) return "image";
    if (getValidColor(item.name)) return "color";
    return "text";
  }
</script>

<div class="clipboard">
  <!-- LEFT: list -->
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
        <!-- NAME + SUBTITLE -->
        <div class="item-body">
          <span class="item-name">{truncate(item.name, 22)}</span>
        </div>
        <div class="type-icon">
          {#if itemType(item) === "image"}
            <img class="icon-thumb" src={item.thumbnail} alt="" />
          {:else if itemType(item) === "color"}
            <div class="icon-swatch" style:background-color={getValidColor(item.name)}></div>
          {/if}
        </div>

      </div>
    {/each}
  </div>

  <!-- RIGHT: preview -->
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
          <div class="text-preview">{activeItem.name}</div>
        {/if}
      </div>

      <div class="metadata">
        <span class="type-badge type-{itemType(activeItem)}">
          {itemType(activeItem)}
        </span>
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

  /* ── LEFT ── */
  .result-list {
    flex: 0 0 224px;
    border-right: 1px solid #333;
    overflow-y: auto;
    padding: 14px 10px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .result-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border-radius: 10px;
    cursor: pointer;
    border: 2px solid transparent;
  }

  .result-item.active {
    background: #2a2a2a;
    border-color: rgba(255,255,255,0.08);
  }

  .type-icon {
    flex-shrink: 0;
    width: 28px;
    height: 28px;
    border-radius: 7px;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .icon-thumb {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .icon-swatch {
    width: 100%;
    height: 100%;
    border-radius: 50%;
  }

  .icon-text {
    width: 100%;
    height: 100%;
    background: #2a2a2a;
    border: 1px solid #3a3a3a;
    border-radius: 7px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #888;
  }

  /* text block */
  .item-body {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .item-name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item-sub {
    font-size: 0.72rem;
    color: #666;
    white-space: nowrap;
  }

  /* ── RIGHT ── */
  .info-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .preview-area {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
    overflow: hidden;
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

  /* ── FOOTER ── */
  .metadata {
    padding: 10px 16px;
    border-top: 1px solid #2a2a2a;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .type-badge {
    font-size: 0.8rem;
    padding: 2px 8px;
    border-radius: 6px;
    text-transform: capitalize;
    font-family: 'JetBrains Mono', monospace;
  }

  .type-badge.type-text   { background: #2a2a2a; color: #888;   border: 1px solid #3a3a3a; }
  .type-badge.type-image  { background: #1a2a1a; color: #6a9;   border: 1px solid #2a3a2a; }
  .type-badge.type-color  { background: #2a1a2a; color: #a6a;   border: 1px solid #3a2a3a; }

  .timestamp {
    margin-left: auto;
    font-size: 0.9rem;
    opacity: 0.4;
  }
</style>
