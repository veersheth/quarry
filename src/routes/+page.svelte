<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import RenderList from "$lib/RenderList.svelte";
  import RenderEmojis from "$lib/RenderEmojis.svelte";
  import {
    query,
    resultItems,
    resultType,
    activeIndex,
  } from "../stores/search";
  import { search } from "../lib/searcher";
  import { handleKeydown } from "../lib/keyHandler";
  import RenderClipboard from "$lib/RenderClipboard.svelte";
  import RenderColorPicker from "$lib/RenderColorPicker.svelte";
  import RenderWebSearch from "$lib/RenderWebSearch.svelte";
  import RenderMath from "$lib/RenderMath.svelte";
  import RenderCamera from "$lib/RenderCamera.svelte";
    import RenderMarkdown from "$lib/RenderMarkdown.svelte";

  let searchInput: HTMLInputElement;
  let appWindow: ReturnType<typeof getCurrentWindow>;
  let isLoading = false;
  let searchTimeout: ReturnType<typeof setTimeout>;

  onMount(() => {
    appWindow = getCurrentWindow();
    const unlisten = appWindow.onFocusChanged(({ payload: focused }) => {
      if (focused && searchInput) searchInput.select();
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  });

  $: if ($query !== undefined) {
    isLoading = true;
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => {
      search($query)
        .then((res) => {
          resultItems.set(res.results);
          resultType.set(res.result_type);
          activeIndex.set(0);
        })
        .catch((err) => {
          console.error("Search error:", err);
          resultItems.set([]);
        })
        .finally(() => {
          isLoading = false;
        });
    }, 100);
  }
</script>

<svelte:window
  on:keydown={(e) =>
    handleKeydown(e, searchInput, activeIndex, resultItems, appWindow)}
/>

<main class="container">
  <div class="panel">
    <!-- svelte-ignore a11y_autofocus -->
    <input
      type="text"
      placeholder="quarry..."
      bind:value={$query}
      bind:this={searchInput}
      autofocus
      class="search"
      class:loading={isLoading}
    />
    <div class="results" class:loading-overlay={isLoading}>
      <div class="results-content" class:dimmed={isLoading}>
        {#if $resultType === "List"}
          <RenderList listitems={$resultItems} {activeIndex} />
        {:else if $resultType === "Grid"}
          <RenderEmojis listitems={$resultItems} {activeIndex} />
        {:else if $resultType === "WebSearch"}
          <RenderWebSearch listitems={$resultItems} {activeIndex} />
        {:else if $resultType === "Markdown"}
          <RenderMarkdown listitems={$resultItems} {activeIndex} />
        {:else if $resultType === "Clipboard"}
          <RenderClipboard listitems={$resultItems} {activeIndex} />
        {:else if $resultType === "ColorPicker"}
          <RenderColorPicker />
        {:else if $resultType === "Home"}
          <RenderList listitems={$resultItems} {activeIndex} />
        {:else if $resultType === "Math"}
          <RenderMath listitems={$resultItems} {activeIndex} />
        {:else if $resultType === "Camera"}
          <RenderCamera />
        {:else}
          Oops
        {/if}
      </div>
    </div>
  </div>
</main>

<style>
  .container {
    display: flex;
    flex: 1;
    height: calc(100vh - 20px);
    flex-direction: column;
    margin: 0;
    padding: 0;
    box-sizing: border-box;
    background-color: rgba(15, 15, 15, 1);
    border: 1px solid rgba(255,255,255,0.35);
    overflow: hidden;
    border-radius: 14px;
  }

  .container * {
    z-index: 0;
    color: #fffffff8;
    font-family: 
      Inter,
      "Segoe UI",
      "Adwaita Sans",
      "Noto Color Emoji",
      sans-serif;
  }

  .panel {
    display: flex;
    flex-direction: column;
    flex: 1;
    max-height: 95vh;
    min-height: 0;
  }

  .search {
    width: 100%;
    display: block;
    padding: 0 20px;
    margin: 0;
    box-sizing: border-box;
    border: none;
    outline: none;
    background: none;
    height: 56px;
    flex-shrink: 0;
    transition: opacity 0.15s ease;

  }

  .search.loading {
    opacity: 0.7;
  }

  .results {
    margin: 0;
    padding: 0;
    border-top: 1px solid rgba(80, 80, 80, 0.7);
    flex: 1;
    box-sizing: border-box;
    overflow-y: auto;
    position: relative;
    min-height: 0;
  }

  .results-content {
    transition: opacity 0.1s ease;
    height: 100%;
  }

  .results-content.dimmed {
    opacity: 0.5;
    pointer-events: none;
  }

  .hint {
    font-size: 11px;
    color: rgba(156, 197, 204, 0.35);
  }
</style>
