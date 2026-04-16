<script lang="ts">
  import { writable, type Writable } from "svelte/store";
  import type { ResultItem } from "../stores/search";
  import { runItemAction } from "./keyHandler";

  function handleClick(item: ResultItem) {
    runItemAction(item);
  }

  export let listitems: {
    name: string;
    actions: { id: string; name: string }[];
    description?: string;
    icon?: string;
  }[] = [];
  export let activeIndex: Writable<number> = writable(0);

  function parseMath(name: string): { query: string; answer: string } {
    const eqIndex = name.lastIndexOf("=");
    if (eqIndex === -1) return { query: name, answer: "" };
    return {
      query: name.slice(0, eqIndex).trim(),
      answer: name.slice(eqIndex + 1).trim(),
    };
  }
</script>

<div class="result-list">
  {#each listitems as item, index}
    {@const { query, answer } = parseMath(item.name)}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      class="result-item"
      class:active={index === $activeIndex}
      on:mouseenter={() => activeIndex.set(index)}
      on:click={() => handleClick(item)}
    >
      <div class="math-answer-bar">
        <span class="math-answer">{answer || item.name}</span>
      </div>

      <div class="math-footer">
        {#if answer}
          <span class="math-query">= {query}</span>
        {/if}
        {#if item.description}
          <span class="math-desc">{item.description}</span>
        {/if}
      </div>
    </div>
  {/each}
</div>

<style>
  .result-item {
  }

  .math-answer-bar {
    font-family: "JetBrainsMono Nerd Font", "JetBrainsMono", "Roboto Mono", monospace;
    width: auto;
    display: flex;
    justify-content: center;
    align-items: center;
    padding: 24px 24px 14px;
    box-sizing: border-box;
  }

  .math-answer {
    background-color: rgba(255,255,255,0.1);
    font-size: 46px;
    color: #f0f0f0;
    border-radius: 12px;
    width: 100%;
    padding: 20px;
    letter-spacing: -0.5px;
    text-align: center;
  }

  .math-footer {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: 10px 22px 14px;
  }

  .math-query {
    font-size: 18px;
    letter-spacing: 0.2px;
    text-align: center;
    margin-bottom: 12px;
  }

  .math-desc {
    font-size: 14px;
    color: rgba(180, 180, 180, 0.55);
    text-align: center;
  }
</style>
