<script lang="ts">
  import { writable, type Writable } from "svelte/store";
  import type { ResultItem } from "../stores/search";
  import { mouseHasMoved } from "../stores/search";
  import FooterBar from "./ui/FooterBar.svelte";
  import Chip from "./ui/Chip.svelte";

  export let listitems: ResultItem[] = [];
  export let activeIndex: Writable<number> = writable(0);
  export let onContextMenu:
    | ((event: MouseEvent, item: ResultItem) => void)
    | undefined = undefined;

  $: activeItem = listitems[$activeIndex];
  $: fontStyle = activeItem ? `font-family: ${fontStack(activeItem.name)}` : "";

  function fontStack(family: string): string {
    const escaped = family.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
    return `"${escaped}", sans-serif`;
  }
</script>

<div class="clipboard">
  <div class="result-list">
    {#each listitems as item, index}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div
        class="result-item"
        class:active={index === $activeIndex}
        data-active={index === $activeIndex}
        on:mouseenter={() => {
          if ($mouseHasMoved) activeIndex.set(index);
        }}
        on:click={() => activeIndex.set(index)}
        on:contextmenu={(event) => {
          event.preventDefault();
          onContextMenu?.(event, item);
        }}
      >
        <div class="type-icon">
          <span class="icon-pill icon-text" style={`font-family: ${fontStack(item.name)}`}>Aa</span>
        </div>
        <div class="item-body">
          <span class="item-name">{item.name}</span>
        </div>
      </div>
    {:else}
      <div class="font-empty">No fonts found</div>
    {/each}
  </div>

  <div class="info-panel">
    {#if activeItem}
      <div class="preview-area">
        <div class="specimen" style={fontStyle}>
          <div class="specimen-heading">{activeItem.name}</div>
          <div class="specimen-large">The quick brown fox jumps over the lazy dog.</div>
          <div class="specimen-medium">Pack my box with five dozen liquor jugs.</div>
          <div class="specimen-alphabet">ABCDEFGHIJKLMNOPQRSTUVWXYZ</div>
          <div class="specimen-alphabet">abcdefghijklmnopqrstuvwxyz</div>
          <div class="specimen-numbers">0123456789 &amp; @!? $%# ([]) &#123;&#125;</div>
        </div>
      </div>
      <FooterBar>
        <Chip mono accent="#a78bfa">font</Chip>
        <Chip>{listitems.length.toLocaleString()} available</Chip>
        <div class="footer-trailing">Press Enter to copy</div>
      </FooterBar>
    {:else}
      <div class="preview-area">
        <div class="font-empty-preview">No installed fonts were found.</div>
      </div>
    {/if}
  </div>
</div>

<style>
  .clipboard {
    display: flex;
    height: 100%;
    color: var(--q-font-color);
  }

  .result-list {
    flex: 0 0 224px;
    border-right: 1px solid rgba(0, 0, 0, 0);
    overflow-y: auto;
    padding: 5px 10px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .result-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border: 1px solid transparent;
    border-radius: var(--q-item-border-radius);
  }

  .result-item.active {
    background: var(--q-active-bg-color);
    border-color: var(--q-active-border-color);
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

  .icon-pill {
    font-size: 1em;
    font-family: var(--q-mono);
    padding: 2px 5px;
    border-radius: 5px;
    letter-spacing: 0.03em;
    white-space: nowrap;
    font-weight: 600;
    text-transform: uppercase;
  }

  .icon-text {
    background: #1e1e1e;
    color: #888;
    border: 1px solid #2e2e2e;
    text-transform: none;
  }

  .item-body {
    flex: 1;
    min-width: 0;
  }

  .item-name {
    display: block;
    overflow: hidden;
    font-size: 0.95em;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .info-panel {
    position: relative;
    display: flex;
    flex: 1;
    flex-direction: column;
    min-width: 0;
  }

  .preview-area {
    display: flex;
    flex: 1;
    align-items: center;
    justify-content: center;
    min-height: 0;
    overflow: hidden;
    padding: 40px 40px 88px;
    box-sizing: border-box;
  }

  .specimen {
    width: min(100%, 680px);
    overflow-wrap: anywhere;
  }

  .specimen-heading {
    margin-bottom: 30px;
    color: var(--q-text-dim);
    font-family: var(--q-mono);
    font-size: 0.82em;
  }

  .specimen-large {
    margin-bottom: 24px;
    font-size: clamp(1.8em, 4vw, 3.4em);
    line-height: 1.18;
  }

  .specimen-medium {
    margin-bottom: 36px;
    color: var(--q-text-muted);
    font-size: clamp(1.1em, 2vw, 1.55em);
    line-height: 1.4;
  }

  .specimen-alphabet {
    margin-top: 10px;
    color: var(--q-text-dim-active);
    font-size: 1.08em;
    letter-spacing: 0.04em;
    overflow-wrap: anywhere;
  }

  .specimen-numbers {
    margin-top: 16px;
    color: var(--q-text-dim);
    font-size: 1.08em;
  }

  .font-empty {
    margin: auto;
    color: var(--q-text-dim);
  }

  .font-empty-preview {
    color: var(--q-text-dim);
  }

  .footer-trailing {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-left: auto;
    color: var(--q-text-dim);
  }
</style>
