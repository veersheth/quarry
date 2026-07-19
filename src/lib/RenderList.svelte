<script lang="ts">
  import { writable, type Writable } from "svelte/store";
  import { convertFileSrc, invoke } from "@tauri-apps/api/core";
  import type { ResultItem } from "../stores/search";
  import { mouseHasMoved } from "../stores/search";
  import { runItemAction } from "./keyHandler";

  function iconSrc(icon: string): string {
    return icon.startsWith("/") ? convertFileSrc(icon) : icon;
  }

  function nameHue(name: string): number {
    let h = 0;
    for (let i = 0; i < name.length; i++) h = (h * 31 + name.charCodeAt(i)) & 0xffff;
    return h % 360;
  }

  function nameInitial(name: string): string {
    return name.trim().charAt(0).toUpperCase() || "?";
  }

  let iconLoaded: Record<string, boolean> = {};
  let iconError: Record<string, boolean> = {};
  function onIconLoad(src: string) { iconLoaded = { ...iconLoaded, [src]: true }; }
  function onIconError(src: string) { iconError = { ...iconError, [src]: true }; }

  export let listitems: {
    name: string;
    actions: { id: string; name: string }[];
    description?: string;
    icon?: string;
    pinned?: boolean;
    draggable_path?: string;
  }[] = [];
  export let activeIndex: Writable<number> = writable(0);
  export let onContextMenu:
    | ((e: MouseEvent, item: (typeof listitems)[number]) => void)
    | undefined = undefined;

  $: {
    listitems;
    mouseHasMoved.set(false);
  }
</script>

<div class="result-list">
  {#each listitems as item, index}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      class="result-item"
      class:active={index === $activeIndex}
      class:has-desc={!!item.description}
      class:pinned={item.pinned}
      data-active={index === $activeIndex}
      on:mouseenter={() => {
        if ($mouseHasMoved) activeIndex.set(index);
      }}
      on:click={() => runItemAction(item)}
      on:contextmenu={(e) => {
        e.preventDefault();
        onContextMenu?.(e, item);
      }}
      on:dragstart|preventDefault
    >
      {#if item.icon}
        {@const src = iconSrc(item.icon)}
        <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
        <div
          class="icon-wrap"
          class:drag-source={!!item.draggable_path}
          on:mousedown|preventDefault={(e) => {
            if (!item.draggable_path || e.button !== 0) return;
            invoke("start_drag", { path: item.draggable_path }).catch(console.error);
          }}
          on:click={(e) => { if (item.draggable_path) e.stopPropagation(); }}
        >
          {#if iconError[src]}
            <div class="icon-avatar" style="--hue: {nameHue(item.name)}">
              {nameInitial(item.name)}
            </div>
          {/if}
          <img
            class="item-icon"
            class:icon-loaded={iconLoaded[src]}
            {src}
            alt=""
            draggable="false"
            on:load={() => onIconLoad(src)}
            on:error={() => onIconError(src)}
          />
        </div>
      {/if}
      <div class="item-text">
        <span class="item-name">{item.name}</span>
        {#if item.description}
          <span class="item-desc">{item.description}</span>
        {/if}
      </div>
      {#if index < 4}
        <span class="alt-hint">Alt + {index + 1}</span>
      {/if}
    </div>
  {/each}
</div>

<style>
  .result-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 5px 0;
  }

  .result-item {
    font-size: 1em;
    display: flex;
    align-items: center;
    width: auto;
    padding: 12px 10px;
    margin: 0 12px;
    border-radius: var(--q-item-border-radius);
    background: none;
    background: var(--q-surface);
    text-align: left;
    color: var(--q-text-secondary);
    border: 2px solid transparent;
  }

  .result-item.pinned {
    border-color: var(--q-pin-border);
  }

  .result-item.active {
    background-color: var(--q-active-bg-color);
    border: 2px solid var(--q-active-border-color);
  }

  .result-item.pinned.active {
    border-color: var(--q-pin-border-active);
  }

  <!-- .result-item.active .icon-wrap { -->
  <!--   filter: drop-shadow(0 0 4px rgba(255, 255, 255, 0.3)); -->
  <!-- } -->

  .icon-wrap {
    position: relative;
    width: 20px;
    height: 20px;
    margin-right: 14px;
    flex-shrink: 0;
  }

  .icon-wrap.drag-source { cursor: grab; }
  .icon-wrap.drag-source:active { cursor: grabbing; }

  .icon-avatar {
    position: absolute;
    inset: 0;
    border-radius: 5px;
    background: hsl(var(--hue), 35%, 22%);
    color: hsl(var(--hue), 60%, 68%);
    font-size: 0.65em;
    font-weight: 700;
    display: flex;
    align-items: center;
    justify-content: center;
    letter-spacing: 0;
  }

  img.item-icon {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: contain;
    object-position: center;
    opacity: 0;
    transition: opacity 0.18s ease;
  }

  img.item-icon.icon-loaded {
    opacity: 1;
  }

  .item-text {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: baseline;
    gap: 0.8rem;
    overflow: hidden;
  }

  .item-name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex-shrink: 1;
    min-width: 0;
  }

  /* Only cap name width when a description is also present */
  .has-desc .item-name {
    flex-shrink: 0;
    max-width: 50%;
  }

  .item-desc {
    opacity: 0.4;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    min-width: 0;
  }

  .alt-hint {
    font-size: 0.8em;
    opacity: 0.5;
    flex-shrink: 0;
    margin-left: 8px;
    font-family: var(--q-mono);
  }
</style>
