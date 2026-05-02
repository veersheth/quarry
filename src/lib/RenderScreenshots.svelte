<script lang="ts">
  import { writable, type Writable } from "svelte/store";
  import { runItemAction } from "./keyHandler";
  import { execute } from "./searcher";
  import { query } from "../stores/search";
  import { get } from "svelte/store";
  import type { ResultItem } from "../stores/search";

  export let listitems: {
    name: string;
    actions: { id: string; name: string }[];
    description?: string;
    thumbnail?: string;
  }[] = [];
  export let activeIndex: Writable<number> = writable(0);
  export let onContextMenu:
    | ((e: MouseEvent, item: (typeof listitems)[number]) => void)
    | undefined = undefined;

  $: active = listitems[$activeIndex];

  // description is packed as "date|dims|size"
  function parseMeta(desc?: string) {
    if (!desc) return { date: "", dims: "", size: "" };
    const [date = "", dims = "", size = ""] = desc.split("|");
    return { date, dims, size };
  }

  function handleClick(item: typeof listitems[0]) {
    runItemAction(item as unknown as ResultItem);
  }

  function truncateName(name: string, max = 22): string {
    if (name.length <= max) return name;
    const ext = name.lastIndexOf(".");
    if (ext > 0) {
      const base = name.slice(0, ext);
      const extension = name.slice(ext);
      return base.slice(0, max - extension.length - 1) + "…" + extension;
    }
    return name.slice(0, max - 1) + "…";
  }
</script>

<div class="screenshots">
  <!-- Left: scrollable thumbnail grid -->
  <div class="grid-panel">
    {#each listitems as item, index}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="grid-item"
        class:active={index === $activeIndex}
        data-active={index === $activeIndex ? "true" : undefined}
        on:mouseenter={() => activeIndex.set(index)}
        on:click={() => handleClick(item)}
        on:contextmenu={(e) => { e.preventDefault(); onContextMenu?.(e, item); }}
      >
        <div class="thumb-wrap">
          {#if item.thumbnail}
            <img class="thumb" src={item.thumbnail} alt={item.name} />
          {:else}
            <div class="thumb-placeholder">IMG</div>
          {/if}
        </div>
        <span class="grid-name">{truncateName(item.name)}</span>
      </div>
    {/each}
  </div>

  <!-- Right: large preview + metadata -->
  <div class="preview-panel">
    {#if active}
      {@const meta = parseMeta(active.description)}
      <div class="preview-image-wrap">
        {#if active.thumbnail}
          <img class="preview-image" src={active.thumbnail} alt={active.name} />
        {:else}
          <div class="preview-placeholder">No preview</div>
        {/if}
      </div>

      <div class="preview-meta">
        <span class="meta-name" title={active.name}>{active.name}</span>
        <div class="meta-chips">
          {#if meta.date}<span class="chip">{meta.date}</span>{/if}
          {#if meta.dims}<span class="chip">{meta.dims}</span>{/if}
          {#if meta.size}<span class="chip">{meta.size}</span>{/if}
        </div>
        <div class="meta-actions">
          {#each active.actions as action}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <button
              class="action-btn"
              class:action-danger={action.name === "Delete"}
              on:click={() => execute(action.id, active.name, get(query))}
            >
              {action.name}
            </button>
          {/each}
        </div>
      </div>
    {:else}
      <div class="empty">no screenshots found</div>
    {/if}
  </div>
</div>

<style>
  .screenshots {
    display: flex;
    height: 100%;
    color: #eee;
    overflow: hidden;
  }

  /* ── Grid panel ─────────────────────────────────────── */
  .grid-panel {
    flex: 0 0 260px;
    border-right: 1px solid #1e1e1e;
    overflow-y: auto;
    padding: 10px 8px;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px;
    align-content: start;
  }

  .grid-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 5px;
    border-radius: 8px;
    border: 1.5px solid transparent;
    cursor: pointer;
    background: #111;
    transition: background 0.1s;
  }

  .grid-item:hover  { background: #181818; }
  .grid-item.active { background: #1e1e26; border-color: rgba(255,255,255,0.12); }

  .thumb-wrap {
    width: 100%;
    aspect-ratio: 16 / 10;
    border-radius: 5px;
    overflow: hidden;
    background: #0d0d0d;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .thumb {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .thumb-placeholder {
    font-size: 0.6rem;
    color: #444;
    font-family: monospace;
    letter-spacing: 0.1em;
  }

  .grid-name {
    font-size: 0.65rem;
    color: #666;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    text-align: center;
    line-height: 1.3;
  }

  .grid-item.active .grid-name { color: #aaa; }

  /* ── Preview panel ──────────────────────────────────── */
  .preview-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    overflow: hidden;
  }

  .preview-image-wrap {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #0a0a0a;
    overflow: hidden;
    min-height: 0;
  }

  .preview-image {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    display: block;
  }

  .preview-placeholder {
    color: #333;
    font-size: 0.85rem;
  }

  .preview-meta {
    flex-shrink: 0;
    padding: 10px 14px;
    border-top: 1px solid #1e1e1e;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .meta-name {
    font-size: 0.8rem;
    color: #bbb;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-family: 'JetBrainsMono Nerd Font', 'Fira Code', monospace;
  }

  .meta-chips {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .chip {
    font-size: 0.68rem;
    font-family: 'JetBrainsMono Nerd Font', 'Fira Code', monospace;
    background: #1a1a1a;
    border: 1px solid #2a2a2a;
    color: #666;
    padding: 2px 8px;
    border-radius: 5px;
  }

  .meta-actions {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .action-btn {
    font-size: 0.72rem;
    padding: 4px 10px;
    border-radius: 6px;
    border: 1px solid #2a2a2a;
    background: #161616;
    color: #aaa;
    cursor: pointer;
    font-family: inherit;
    transition: background 0.1s, color 0.1s;
  }

  .action-btn:hover { background: #222; color: #eee; border-color: #444; }

  .action-btn.action-danger       { color: #f87171; border-color: rgba(248,113,113,0.2); }
  .action-btn.action-danger:hover { background: rgba(248,113,113,0.1); border-color: rgba(248,113,113,0.5); }

  .empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.85rem;
    color: #333;
  }
</style>
