<script lang="ts">
  import { writable, type Writable } from "svelte/store";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import type { ResultItem } from "../stores/search";
  import { mouseHasMoved } from "../stores/search";
  import { runItemAction } from "./keyHandler";
  import RenderList from "./RenderList.svelte";

  export let listitems: ResultItem[] = [];
  export let activeIndex: Writable<number> = writable(0);
  export let onContextMenu:
    | ((e: MouseEvent, item: ResultItem) => void)
    | undefined = undefined;

  $: widgets = listitems.filter(i => i.group === "widget");
  $: apps    = listitems.filter(i => i.group !== "widget");
  $: cols    = Math.min(widgets.length, 4);


  function iconSrc(icon: string): string {
    return icon.startsWith("/") ? convertFileSrc(icon) : icon;
  }

  let iconLoaded: Record<string, boolean> = {};
  let iconError:  Record<string, boolean> = {};
  function onLoad(src: string)  { iconLoaded = { ...iconLoaded, [src]: true }; }
  function onError(src: string) { iconError  = { ...iconError,  [src]: true }; }

  $: { listitems; mouseHasMoved.set(false); }
</script>

{#if widgets.length > 0}
  <div class="widget-row" style="grid-template-columns: repeat({cols}, 1fr)">
    {#each widgets as item, i}
      <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
      <div
        class="widget"
        class:active={i === $activeIndex}
        data-active={i === $activeIndex}
        on:mouseenter={() => { if ($mouseHasMoved) activeIndex.set(i); }}
        on:click={() => runItemAction(item)}
        on:contextmenu={(e) => { e.preventDefault(); onContextMenu?.(e, item); }}
      >
        {#if item.icon}
          {@const src = iconSrc(item.icon)}
          <div class="widget-icon">
            <img
              class="widget-img"
              class:loaded={iconLoaded[src]}
              {src} alt=""
              on:load={() => onLoad(src)}
              on:error={() => onError(src)}
            />
          </div>
        {/if}
        <div class="widget-text">
          <span class="widget-name">{item.name}</span>
          {#if item.description}
            <span class="widget-desc">{item.description}</span>
          {/if}
        </div>
      </div>
    {/each}
  </div>
{/if}

{#if apps.length > 0}
  <RenderList
    listitems={apps}
    {activeIndex}
    indexOffset={widgets.length}
    {onContextMenu}
  />
{/if}

<style>
  .widget-row {
    display: grid;
    gap: 8px;
    padding: 16px 12px 10px;
  }

  .widget {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 14px 18px;
    min-width: 0;
    overflow: hidden;
    border-radius: var(--q-item-border-radius);
    border: 1px solid var(--q-border-subtle);
    color: var(--q-text-secondary);
    cursor: default;
    box-sizing: border-box;
  }

  .widget.active {
    background: var(--q-active-bg-color);
    color: var(--q-font-color, #fff);
  }

  .widget-icon {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
    position: relative;
  }

  .widget-img {
    width: 100%;
    height: 100%;
    object-fit: contain;
    opacity: 0;
    transition: opacity 0.15s ease;
  }

  .widget-img.loaded { opacity: 1; }

  .widget-text {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }

  .widget-name {
    font-size: var(--q-font-size);
    font-family: var(--q-sans);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: inherit;
  }

  .widget-desc {
    font-size: 0.8em;
    font-family: var(--q-sans);
    color: var(--q-text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
