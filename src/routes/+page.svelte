<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount, tick } from "svelte";
  import ResultsList from "../lib/ResultsList.svelte";
  import ResultsGrid from "$lib/ResultsGrid.svelte";

  type ResultItem = {
    name: string;
    exec: string;
    description?: string;
    icon?: string;
  };

  type ResultType = "List" | "Grid";

  type SearchResult = {
    results: ResultItem[];
    result_type: ResultType;
  };

  let query: string = "";
  let resultItems: ResultItem[] = [];
  let activeIndex: number = 0;
  let searchInput: HTMLInputElement;
  let resultType: ResultType = "List";

  let appWindow: ReturnType<typeof getCurrentWindow>;

  // Cache setup
  const searchCache = new Map<string, SearchResult>();
  const CACHE_TTL = 5 * 60 * 1000; // 5 minutes
  const cacheTimestamps = new Map<string, number>();

  onMount(() => {
    appWindow = getCurrentWindow();

    // Listen for when window becomes visible
    const unlisten = appWindow.onFocusChanged(({ payload: focused }) => {
      if (focused && searchInput) {
        // Select all text when window gains focus
        searchInput.select();
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  });
  async function execute(executable: string) {
    try {
      await invoke("execute", { executable });
      if (appWindow) {
        await appWindow.hide();
      }
    } catch (e) {
      console.error("Execute error:", e);
    }
  }

  async function search() {
    const cached = searchCache.get(query);
    const timestamp = cacheTimestamps.get(query);

    if (cached && timestamp && Date.now() - timestamp < CACHE_TTL) {
      resultItems = cached.results;
      resultType = cached.result_type;
      activeIndex = 0;
      return;
    }

    try {
      const searchResult = await invoke<SearchResult>("search", { query });

      // add 2 cache
      searchCache.set(query, searchResult);
      cacheTimestamps.set(query, Date.now());

      resultItems = searchResult.results;
      resultType = searchResult.result_type;
      activeIndex = 0;
    } catch (e) {
      console.error(e);
      resultItems = [
        { name: "error error quarry error", exec: "notify-send 'Error'" },
      ];
      activeIndex = 0;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (searchInput && document.activeElement !== searchInput) {
      searchInput.focus();
    }

    if (event.key === "Escape" || (event.key === "u" && event.ctrlKey)) {
      event.preventDefault();
      if (appWindow) {
        appWindow.hide();
      }
      return;
    }

    if (event.key === "w" && event.ctrlKey) {
      event.preventDefault();
      query = query.replace(/\s+$/, "");
      query = query.replace(/\S+$/, "");
    }

    if (resultItems.length === 0) return;

    if (event.key === "Tab" && !event.shiftKey) {
      event.preventDefault();
      activeIndex = (activeIndex + 1) % resultItems.length;
    } else if (event.key === "Tab" && event.shiftKey) {
      event.preventDefault();
      activeIndex =
        activeIndex === 0 ? resultItems.length - 1 : activeIndex - 1;
    } else if (event.key === "n" && event.ctrlKey) {
      event.preventDefault();
      activeIndex = (activeIndex + 1) % resultItems.length;
    } else if (event.key === "p" && event.ctrlKey) {
      event.preventDefault();
      activeIndex =
        activeIndex === 0 ? resultItems.length - 1 : activeIndex - 1;
    } else if (event.key === "Enter") {
      event.preventDefault();
      execute(resultItems[activeIndex].exec);
    }
  }

  $: if (query !== undefined) search();
</script>

<svelte:window on:keydown={handleKeydown} />

<main class="container">
  <div class="panel">
    <!-- svelte-ignore a11y_autofocus -->
    <input
      type="text"
      placeholder="Search…"
      bind:value={query}
      bind:this={searchInput}
      autofocus
      class="search"
    />
    <div class="results">
      {#if resultType === "List"}
        <ResultsList listitems={resultItems} {activeIndex} />
      {:else if resultType === "Grid"}
        <ResultsGrid listitems={resultItems} {activeIndex} />
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
    margin: 0px;
    padding: 0;
    box-sizing: border-box;
    background-color: rgba(20, 20, 20, 0.8);
    border: 1px solid rgba(80, 80, 80, 1);
    overflow: hidden;
    border-radius: 14px;
    * {
      color: #fffffff8;
      font-family:
        Segoe UI,
        Inter,
        Adwaita Sans,
        sans-serif;
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
    overflow-y: auto;
    box-sizing: border-box;
  }
</style>
