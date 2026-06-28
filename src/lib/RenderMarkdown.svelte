<script lang="ts">
  import { writable, type Writable } from "svelte/store";
  import { marked } from 'marked';

  export let listitems: {
    name: string;
    actions: { id: string; name: string }[];
    description?: string;
  }[] = [];
  export let activeIndex: Writable<number> = writable(0);

  function renderMarkdown(content: string | undefined): string {
    if (!content) return "";
    return marked.parse(content) as string;
  }
</script>

<div class="markdown-container">
  {#each listitems as item, index}
    {#if index === $activeIndex}
      <div class="markdown-body">
        {@html renderMarkdown(item.description)}
      </div>
    {/if}
  {/each}
</div>

<style>
  .markdown-container {
    padding: 20px;
    height: 100%;
    overflow-y: auto;
    color: rgba(255, 255, 255, 0.9);
    font-family: Georgia, 'Times New Roman', serif;
    line-height: 1.6;
    font-size: var(--q-font-size, 15px);
  }

  .markdown-body :global(h1) {
    font-size: 1.8em;
    font-weight: 600;
    margin: 0 0 4px 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    padding-bottom: 8px;
  }

  .markdown-body :global(h2) {
    font-size: 1.4em;
    font-weight: 600;
    margin: 20px 0 8px 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    padding-bottom: 6px;
  }

  .markdown-body :global(h3) {
    font-size: 1em;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: rgba(255, 255, 255, 0.4);
    margin: 20px 0 8px 0;
  }

  .markdown-body :global(h4),
  .markdown-body :global(h5),
  .markdown-body :global(h6) {
    font-size: 0.9em;
    font-weight: 500;
    margin: 16px 0 6px 0;
    color: rgba(255, 255, 255, 0.6);
  }

  .markdown-body :global(p) {
    margin: 0 0 12px 0;
  }

  .markdown-body :global(ol),
  .markdown-body :global(ul) {
    padding-left: 1.4em;
    margin: 0 0 12px 0;
  }

  .markdown-body :global(li) {
    margin-bottom: 6px;
    color: rgba(255, 255, 255, 0.85);
  }

  .markdown-body :global(li > ul),
  .markdown-body :global(li > ol) {
    margin: 6px 0 0 0;
  }

  .markdown-body :global(blockquote) {
    margin: 4px 0 12px 0;
    padding: 0 0 0 12px;
    border-left: 2px solid rgba(255, 255, 255, 0.2);
    color: rgba(255, 255, 255, 0.45);
    font-style: italic;
  }

  .markdown-body :global(blockquote p) {
    margin: 0;
  }

  .markdown-body :global(code) {
    font-family: 'JetBrainsMono Nerd Font', 'Fira Code', 'Cascadia Mono', monospace;
    font-size: 0.85em;
    background: rgba(255, 255, 255, 0.1);
    padding: 0.15em 0.4em;
    border-radius: 4px;
  }

  .markdown-body :global(pre) {
    background: rgba(0, 0, 0, 0.3);
    padding: 14px 16px;
    border-radius: 8px;
    overflow-x: auto;
    margin: 0 0 14px 0;
  }

  .markdown-body :global(pre code) {
    background: none;
    padding: 0;
    font-size: 0.85em;
  }

  .markdown-body :global(table) {
    width: 100%;
    border-collapse: collapse;
    margin: 0 0 16px 0;
    font-size: 0.9em;
  }

  .markdown-body :global(th) {
    text-align: left;
    padding: 8px 12px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.2);
    color: rgba(255, 255, 255, 0.5);
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    font-size: 0.8em;
  }

  .markdown-body :global(td) {
    padding: 8px 12px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.07);
    color: rgba(255, 255, 255, 0.85);
    vertical-align: top;
  }

  .markdown-body :global(tr:last-child td) {
    border-bottom: none;
  }

  .markdown-body :global(hr) {
    border: none;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
    margin: 20px 0;
  }

  .markdown-body :global(a) {
    color: rgba(255, 255, 255, 0.7);
    text-decoration: underline;
    text-decoration-color: rgba(255, 255, 255, 0.3);
  }

  .markdown-body :global(a:hover) {
    color: rgba(255, 255, 255, 0.95);
  }

  .markdown-body :global(strong) {
    font-weight: 700;
    color: rgba(255, 255, 255, 1);
  }

  .markdown-body :global(em) {
    font-style: italic;
    color: rgba(255, 255, 255, 0.75);
  }

  .markdown-body :global(img) {
    max-width: 100%;
    border-radius: 6px;
  }
</style>
