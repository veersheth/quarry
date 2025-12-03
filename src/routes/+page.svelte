<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import RenderList from "$lib/RenderList.svelte";
  import RenderGrid from "$lib/RenderGrid.svelte";
  import RenderDictionary from "$lib/RenderDictionary.svelte";
  import { query, resultItems, resultType, activeIndex } from "../stores/search";
  import { search } from "../lib/searcher";
  import { handleKeydown } from "../lib/keyHandler";

  let searchInput: HTMLInputElement;
  let appWindow: ReturnType<typeof getCurrentWindow>;

  onMount(() => {
    appWindow = getCurrentWindow();

    const unlisten = appWindow.onFocusChanged(({ payload: focused }) => {
      if (focused && searchInput) searchInput.select();
    });

    return () => {
      unlisten.then(fn => fn());
    };
  });

  $: if ($query !== undefined) {
    search($query).then(res => {
      resultItems.set(res.results);
      resultType.set(res.result_type);
      activeIndex.set(0);
    });
  }
</script>

<svelte:window on:keydown={(e) => handleKeydown(e, searchInput, activeIndex, resultItems, appWindow)} />

<main class="container">
  <div class="panel">
    <input type="text" placeholder="Search…" bind:value={$query} bind:this={searchInput} autofocus class="search" />
    <div class="results">

      {#if $resultType === "List"}
        <RenderList listitems={$resultItems} activeIndex={activeIndex} />
      {:else if $resultType === "Grid"}
        <RenderGrid listitems={$resultItems} activeIndex={activeIndex} />
      {:else if $resultType === "Dictionary"}
        <RenderDictionary listitems={$resultItems} activeIndex={activeIndex} />
      {:else}
        Oops
      {/if}

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
    background-color: rgba(20, 20, 20, 1);
    border: 1px solid rgba(80, 80, 80, 1);
    overflow: hidden;
    border-radius: 14px;
    * {
      color: #fffffff8;
      font-family: Segoe UI, Inter, Adwaita Sans, sans-serif;
    }
  }

  .panel {
    display: flex;
    flex-direction: column;
    flex: 1;
    max-height: 95vh;
  }

  .search {
    width: 100%;
    display: block;
    padding: 20px;
    margin: 0;
    box-sizing: border-box;
    border: none;
    outline: none;
    background: none;
    height: 50px;
  }

  .results {
    margin: 0;
    padding: 0;
    border-top: 1px solid rgba(80, 80, 80, 1);
    padding-top: 10px;
    flex: 1;
    box-sizing: border-box;
    overflow-y: auto;
  }
</style>

