<script lang="ts">
  import { writable, get, type Writable } from "svelte/store";
  import { runItemAction } from "./keyHandler";
  import { execute } from "./searcher";
  import { query, mouseHasMoved } from "../stores/search";
  import type { ResultItem } from "../stores/search";
  import FooterBar from "./ui/FooterBar.svelte";
  import Chip from "./ui/Chip.svelte";
  import Timestamp from "./ui/Timestamp.svelte";

  export let listitems: ResultItem[] = [];
  export let activeIndex: Writable<number> = writable(0);
  export let onContextMenu:
    | ((e: MouseEvent, item: ResultItem) => void)
    | undefined = undefined;

  $: active = listitems[$activeIndex];

  function parseMeta(desc?: string) {
    if (!desc) return { date: "", dims: "", size: "" };
    const [date = "", dims = "", size = ""] = desc.split("|");
    return { date, dims, size };
  }

  function truncateName(name: string, max = 20): string {
    if (name.length <= max) return name;
    const dot = name.lastIndexOf(".");
    if (dot > 0) {
      const ext = name.slice(dot);
      return name.slice(0, max - ext.length - 1) + "…" + ext;
    }
    return name.slice(0, max - 1) + "…";
  }
</script>

<div class="screenshots">
  <div class="grid-panel">
    {#each listitems as item, index}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="grid-item"
        class:active={index === $activeIndex}
        data-active={index === $activeIndex ? "true" : undefined}
        on:mouseenter={() => { if ($mouseHasMoved) activeIndex.set(index); }}
        on:click={() => runItemAction(item)}
        on:contextmenu={(e) => {
          e.preventDefault();
          onContextMenu?.(e, item);
        }}
      >
        <div class="thumb-wrap">
          {#if item.thumbnail}
            <img class="thumb" src={item.thumbnail} alt="" />
          {:else}
            <div class="thumb-placeholder">IMG</div>
          {/if}
        </div>
        <span class="item-name">{truncateName(item.name)}</span>
      </div>
    {/each}

    {#if listitems.length === 0}
      <div class="empty">no screenshots found</div>
    {/if}
  </div>

  {#if active}
    {@const meta = parseMeta(active.description)}
    <FooterBar>
      {#if meta.dims}<Chip>{meta.dims}</Chip>{/if}
      {#if meta.size}<Chip>{meta.size}</Chip>{/if}
      {#if meta.date}<Timestamp value={meta.date} />{/if}
    </FooterBar>
  {/if}
</div>

<style>
  .screenshots {
    display: flex;
    flex-direction: column;
    height: 100%;
    color: var(--q-font-color);
  }

  .grid-panel {
    flex: 1;
    overflow-y: auto;
    padding: 10px 10px 6px;
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: 6px;
    align-content: start;
    min-height: 0;
  }

  .grid-item {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 12px;
    border-radius: var(--q-item-border-radius);
    border: 1.5px solid transparent;
    transition: transform 120ms ease, background 80ms ease;
  }

  .grid-item:hover {
    background: rgba(255, 255, 255, 0.03);
    transform: scale(1.04);
  }
  .grid-item.active {
    background: var(--q-active-bg-color);
    border-color: var(--q-active-border-color);
    transform: scale(1.04);
  }

  .thumb-wrap {
    width: 100%;
    aspect-ratio: 16 / 10;
    border-radius: 6px;
    overflow: hidden;
    background: var(--q-thumb-bg);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .thumb {
    width: 100%;
    height: 100%;
    object-fit: none;
    display: block;
  }

  .thumb-placeholder {
    font-size: 0.6em;
    color: var(--q-text-placeholder);
    letter-spacing: 0.1em;
  }

  .item-name {
    display: block;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 0.75em;
    color: var(--q-text-dim);
    text-align: center;
  }

  .grid-item.active .item-name {
    color: var(--q-text-dim-active);
  }

  .empty {
    grid-column: 1 / -1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 40px 0;
    font-size: 0.85em;
    color: var(--q-text-empty);
  }
</style>
