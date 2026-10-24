<script lang="ts">
  import { onMount } from "svelte";
  import { writable, type Writable } from "svelte/store";
  import { listen } from "@tauri-apps/api/event";
  import { marked } from 'marked';

  export let listitems: {
    name: string;
    actions: { id: string; name: string }[];
    description?: string;
  }[] = [];
  export let activeIndex: Writable<number> = writable(0);

  type DictionaryProgress = {
    stage: string;
    downloadedBytes: number;
    totalBytes: number | null;
    indexedEntries: number;
    totalEntries: number;
    message: string;
  };

  let dictionaryProgress: DictionaryProgress | null = null;
  let clearProgressTimer: ReturnType<typeof setTimeout> | undefined;
  $: progressPercent = dictionaryProgress?.totalBytes
    ? Math.min(100, dictionaryProgress.downloadedBytes / dictionaryProgress.totalBytes * 100)
    : 0;

  onMount(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;

    listen<DictionaryProgress>("quarry-dictionary-progress", ({ payload }) => {
      dictionaryProgress = payload;
      clearTimeout(clearProgressTimer);
      if (payload.stage === "complete") {
        clearProgressTimer = setTimeout(() => {
          dictionaryProgress = null;
        }, 1800);
      }
    }).then((stopListening) => {
      if (cancelled) stopListening();
      else unlisten = stopListening;
    });

    return () => {
      cancelled = true;
      unlisten?.();
      clearTimeout(clearProgressTimer);
    };
  });

  function renderMarkdown(content: string | undefined): string {
    if (!content) return "";
    return marked.parse(content) as string;
  }

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    const units = ["KB", "MB", "GB"];
    let value = bytes / 1024;
    let unit = 0;
    while (value >= 1024 && unit < units.length - 1) {
      value /= 1024;
      unit += 1;
    }
    return `${value.toFixed(1)} ${units[unit]}`;
  }
</script>

<div class="markdown-container">
  {#if dictionaryProgress}
    <section
      class="dictionary-progress"
      class:failed={dictionaryProgress.stage === "failed"}
      class:complete={dictionaryProgress.stage === "complete"}
      aria-live="polite"
    >
      <div class="progress-message">{dictionaryProgress.message}</div>
      {#if dictionaryProgress.stage !== "failed" && dictionaryProgress.stage !== "complete"}
        <div
          class="progress-track"
          role="progressbar"
          aria-label="Dictionary setup progress"
          aria-valuemin="0"
          aria-valuemax="100"
          aria-valuenow={Math.round(progressPercent)}
        >
          <div
            class="progress-fill"
            class:indeterminate={!dictionaryProgress.totalBytes}
            style={`width:${progressPercent}%`}
          ></div>
        </div>
        <div class="progress-details">
          {#if dictionaryProgress.totalBytes}
            {formatBytes(dictionaryProgress.downloadedBytes)} of {formatBytes(dictionaryProgress.totalBytes)} downloaded
          {:else if dictionaryProgress.stage === "connecting"}
            Connecting to dictionary source…
          {:else}
            {formatBytes(dictionaryProgress.downloadedBytes)} downloaded
          {/if}
          · {dictionaryProgress.indexedEntries.toLocaleString()} of {dictionaryProgress.totalEntries.toLocaleString()} entries indexed
        </div>
      {/if}
    </section>
  {/if}

  {#each listitems as item, index}
    {#if index === $activeIndex}
      {#if !(dictionaryProgress && item.name === "Preparing the local dictionary")}
        <div class="markdown-body">
          {@html renderMarkdown(item.description)}
        </div>
      {/if}
    {/if}
  {/each}
</div>

<style>
  .dictionary-progress {
    margin: 4px 0 20px;
    padding: 16px 18px;
    border: 1px solid var(--q-border-medium);
    border-radius: 12px;
    background: var(--q-surface, rgba(255, 255, 255, 0.035));
  }

  .progress-message {
    margin-bottom: 10px;
    color: var(--q-text-secondary, rgba(255, 255, 255, 0.8));
  }

  .progress-track {
    height: 6px;
    overflow: hidden;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.1);
  }

  .progress-fill {
    height: 100%;
    border-radius: inherit;
    background: var(--q-active-border-color, #8ab4f8);
    transition: width 180ms ease;
  }

  .progress-fill.indeterminate {
    width: 35% !important;
    animation: dictionary-progress 1.2s ease-in-out infinite alternate;
  }

  .progress-details {
    margin-top: 8px;
    color: var(--q-text-muted, rgba(255, 255, 255, 0.48));
    font-size: 0.82em;
  }

  .dictionary-progress.failed {
    border-color: rgba(248, 113, 113, 0.45);
  }

  .dictionary-progress.complete {
    border-color: rgba(74, 222, 128, 0.35);
  }

  @keyframes dictionary-progress {
    from { transform: translateX(-30%); }
    to { transform: translateX(210%); }
  }

  @media (prefers-reduced-motion: reduce) {
    .progress-fill { transition: none; }
    .progress-fill.indeterminate { animation: none; width: 100% !important; }
  }

  .markdown-container {
    box-sizing: border-box;
    padding: 20px;
    height: 100%;
    min-height: 0;
    overflow-y: auto;
    color: rgba(255, 255, 255, 0.9);
    font-family: var(--q-sans, system-ui, sans-serif);
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
