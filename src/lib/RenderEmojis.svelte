<script lang="ts">
  import { writable, type Writable } from "svelte/store";
  import { runItemAction } from "./keyHandler";

  export let listitems: {
    name: string;
    actions: { id: string; name: string }[];
    description?: string;
    icon?: string;
    pinned?: boolean;
    group?: string;
  }[] = [];

  export let activeIndex: Writable<number> = writable(0);
  export let onContextMenu: ((e: MouseEvent, item: (typeof listitems)[number]) => void) | undefined = undefined;

  type ResultItem = (typeof listitems)[number];

  $: featured = listitems.filter(i => i.group === "featured");
  $: all = listitems.filter(i => i.group !== "featured");

  // Global indices: featured items come first in listitems, so their index is their position.
  // All items start after featured.
  $: featuredOffset = 0;
  $: allOffset = featured.length;

  let mouseHasMoved = false;
  $: { listitems; mouseHasMoved = false; }

  function handleClick(item: ResultItem) {
    runItemAction(item);
  }
</script>

<svelte:window on:mousemove={() => { mouseHasMoved = true; }} />

<div class="emoji-layout">
  {#if featured.length > 0}
    <div class="featured-strip">
      {#each featured as item, i}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div
          class="featured-item"
          class:active={featuredOffset + i === $activeIndex}
          class:pinned={item.pinned}
          data-active={featuredOffset + i === $activeIndex}
          on:mouseenter={() => { if (mouseHasMoved) activeIndex.set(featuredOffset + i); }}
          on:click={() => handleClick(item)}
          on:contextmenu={(e) => { e.preventDefault(); onContextMenu?.(e, item); }}
        >
          <span class="emoji">{item.name}</span>
        </div>
      {/each}
    </div>
    <div class="section-divider"></div>
  {/if}

  <div class="result-grid">
    {#each all as item, i}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div
        class="grid-item"
        class:active={allOffset + i === $activeIndex}
        class:pinned={item.pinned}
        data-active={allOffset + i === $activeIndex}
        on:mouseenter={() => { if (mouseHasMoved) activeIndex.set(allOffset + i); }}
        on:click={() => handleClick(item)}
        on:contextmenu={(e) => { e.preventDefault(); onContextMenu?.(e, item); }}
      >
        <span class="emoji">{item.name}</span>
      </div>
    {/each}
  </div>
</div>

<style>
  .emoji-layout {
    display: flex;
    flex-direction: column;
    min-height: 0;
    padding: 10px;
    height: 95%;
  }

  .featured-strip {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding: 10px 10px 8px;
    flex-shrink: 0;
  }

  .featured-item {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 48px;
    height: 48px;
    border-radius: var(--q-item-border-radius);
    border: 1px solid rgba(60, 60, 60, 0.3);
    cursor: pointer;
    flex-shrink: 0;
  }

  .featured-item.pinned {
    border-color: rgba(168, 85, 247, 0.55);
  }

  .featured-item.active {
    background-color: var(--q-active-bg-color);
    border-color: var(--q-active-border-color);
    box-shadow: 0 0 12px 2px rgba(50, 50, 50, 1);
  }

  .featured-item.pinned.active {
    border-color: rgba(192, 132, 252, 0.8);
  }

  .section-divider {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 10px 6px;
    margin: 10px;
    flex-shrink: 0;
  }

  /* ── Full grid ── */
  .result-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(90px, 1fr));
    gap: 12px;
    padding: 4px 10px 10px;
    overflow-y: auto;
    min-height: 0;
  }

  .grid-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 28px 16px;
    border-radius: var(--q-item-border-radius);
    border: 1px solid rgba(60, 60, 60, 0.3);
    cursor: pointer;
    text-align: center;
    transition: transform 100ms ease, border-radius 100ms ease;
  }

  .grid-item.pinned {
    border-color: rgba(168, 85, 247, 0.45);
  }

  .grid-item.active {
    background-color: var(--q-active-bg-color);
    box-shadow: 0 0 20px 4px rgba(50, 50, 50, 1);
    border: 1px solid var(--q-active-border-color);
  }

  .grid-item.pinned.active {
    border-color: rgba(192, 132, 252, 0.8);
  }

  .emoji {
    font-size: 2rem;
    line-height: 1;
    margin-bottom: 6px;
  }

  .featured-item .emoji {
    font-size: 1.5rem;
    margin-bottom: 0;
  }
</style>
