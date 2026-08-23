<script lang="ts">
  import { writable, type Writable } from "svelte/store";
  import type { ResultItem } from "../stores/search";
  import { mouseHasMoved } from "../stores/search";
  import { runItemAction } from "./keyHandler";

  export let listitems: ResultItem[] = [];
  export let activeIndex: Writable<number> = writable(0);
  export let onContextMenu: ((e: MouseEvent, item: ResultItem) => void) | undefined = undefined;

  function parseMath(name: string): { expr: string; answer: string } {
    const eqIndex = name.lastIndexOf("=");
    if (eqIndex === -1) return { expr: "", answer: name };
    return {
      expr:   name.slice(0, eqIndex).trim(),
      answer: name.slice(eqIndex + 1).trim(),
    };
  }
</script>

<div class="result-list">
  {#each listitems as item, index}
    {@const { expr, answer } = parseMath(item.name)}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      class="result-item"
      class:active={index === $activeIndex}
      data-active={index === $activeIndex}
      on:mouseenter={() => { if ($mouseHasMoved) activeIndex.set(index); }}
      on:click={() => runItemAction(item)}
      on:contextmenu={(e) => { e.preventDefault(); onContextMenu?.(e, item); }}
    >
      <div class="math-answer-bar">
        <span class="math-answer">{answer}</span>
      </div>

      {#if expr}
        <div class="math-footer">
          <span class="math-expr">{expr}</span>
          <span class="math-hint">↵ copy</span>
        </div>
      {/if}
    </div>
  {/each}
</div>

<style>
  .result-list {
    display: flex;
    flex-direction: column;
  }

  .result-item {
    cursor: default;
    border-radius: var(--q-item-border-radius);
    margin: 8px 12px 4px;
    border: 2px solid transparent;
    overflow: hidden;
  }

  .result-item.active {
    border-color: var(--q-active-border-color);
  }

  .math-answer-bar {
    font-family: var(--q-mono);
    display: flex;
    justify-content: center;
    align-items: center;
    padding: 20px 20px 16px;
    box-sizing: border-box;
  }

  .math-answer {
    background-color: var(--q-surface-subtle);
    font-size: 2.8em;
    color: var(--q-font-color);
    border-radius: 10px;
    width: 100%;
    padding: 18px 24px;
    letter-spacing: -0.5px;
    text-align: center;
    box-sizing: border-box;
  }

  .math-footer {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 16px;
    padding: 6px 24px 16px;
  }

  .math-expr {
    font-family: var(--q-mono);
    font-size: 1em;
    color: var(--q-text-secondary);
    letter-spacing: 0.2px;
  }

  .math-hint {
    font-size: 0.78em;
    color: var(--q-text-dim);
    font-family: var(--q-mono);
    background: var(--q-surface-subtle);
    padding: 2px 7px;
    border-radius: 5px;
    border: 1px solid var(--q-border-subtle);
  }
</style>
