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
  
  let showFallback = false;
  
  function truncate(str: string | undefined, maxLength: number): string {
    if (!str) return "";
    return str.length > maxLength ? str.slice(0, maxLength) + "…" : str;
  }
  function handleIconError(event: Event) {
    showFallback = true;
  }
  $: item = listitems[0];
  $: if (item) {
    showFallback = !item.icon;
  }
</script>
{#if item}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    class="result-item"
    class:active={0 === $activeIndex}
    on:mouseenter={() => activeIndex.set(0)}
    on:click={() => handleClick(item)}
  >
    <div class="item-content">
      <span class="item-name">{item.name}</span>
      {#if item.description}
        <div class="item-desc">{truncate(item.description, 70)}</div>
      {/if}
    </div>
    {#if item.icon && !showFallback}
      <img
        class="item-icon"
        src={item.icon}
        alt=""
        on:error={handleIconError}
      />
    {:else}
      <svg class="item-icon fallback-icon" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
        <circle cx="10" cy="10" r="6" stroke="currentColor" stroke-width="2"/>
        <path d="M14.5 14.5L20 20" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      </svg>
    {/if}
  </div>
{/if}
<style>
  .result-item {
    position: relative;
    display: flex;
    height: 100%;
    flex-direction: row;
    align-items: flex-start;
    justify-content: space-between;
    padding: 1rem 1.2rem;
    border-radius: 12px;
    color: #e5e5e5;
    overflow: hidden;
    box-sizing: border-box;
    transition:
      background 120ms ease,
      transform 120ms ease;
  }
  .item-content {
    display: flex;
    flex-direction: column;
    flex: 1;
  }
  img.item-icon,
  svg.item-icon {
    width: 45%;
    height: auto;
    opacity: 0.15;
    object-fit: contain;
    position: absolute;
    right: -30px;
    bottom: -30px;
    pointer-events: none;
    transform: rotate(-15deg) scale(1.15);
    filter: drop-shadow(0 0 30px rgba(66, 133, 244, 0.15))
      drop-shadow(0 0 60px rgba(66, 133, 244, 0.08));
  }
  
  svg.fallback-icon {
    color: #4285f4;
    opacity: 0.12;
  }
 
  .item-name {
    font-size: 1.4rem;
    font-weight: 500;
  }
  .item-desc {
    margin-top: 6px;
    opacity: 0.55;
    font-size: 0.9rem;
    line-height: 1.35;
  }
</style>
