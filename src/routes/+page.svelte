<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import ResultsList from "../lib/ResultsList.svelte";
  type ListItem = {
    name: string;
    exec: string;
    description?: string;
    icon?: string;
  };
  let query: string = "";
  let listitems: ListItem[] = [];
  let activeIndex: number = 0;

  async function execute(executable: string) {
    try {
      await invoke("execute", { executable });
    } catch (e) {
      console.error(e);
    }
  }
  async function search() {
    try {
      listitems = await invoke<ListItem[]>("search", { query });
      activeIndex = 0; // reset to first item on new search
    } catch (e) {
      console.error(e);
      listitems = [
        { name: "Error fetching results", exec: "notify-send 'Error'" },
      ];
      activeIndex = 0;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      query = "";
      return;
    }

    if (event.key === "w" && event.ctrlKey) {
      event.preventDefault();
      // Delete last word
      const trimmed = query.trimEnd();
      const lastSpaceIndex = trimmed.lastIndexOf(" ");
      query = lastSpaceIndex === -1 ? "" : trimmed.substring(0, lastSpaceIndex);
      return;
    }

    if (listitems.length === 0) return;

    if (event.key === "Tab" && !event.shiftKey) {
      event.preventDefault();
      activeIndex = (activeIndex + 1) % listitems.length;
    } else if (event.key === "Tab" && event.shiftKey) {
      event.preventDefault();
      activeIndex = activeIndex === 0 ? listitems.length - 1 : activeIndex - 1;
    } else if (event.key === "n" && event.ctrlKey) {
      event.preventDefault();
      activeIndex = (activeIndex + 1) % listitems.length;
    } else if (event.key === "p" && event.ctrlKey) {
      event.preventDefault();
      activeIndex = activeIndex === 0 ? listitems.length - 1 : activeIndex - 1;
    } else if (event.key === "Enter") {
      event.preventDefault();
      execute(listitems[activeIndex].exec);
    }
  }

  $: if (query !== undefined) search();
</script>

<svelte:window on:keydown={handleKeydown} />

<main class="container">
  <div class="panel">
    <input
      type="text"
      placeholder="Search…"
      bind:value={query}
      class="search"
    />
    <div class="results">
      <ResultsList {listitems} {activeIndex} />
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
    overflow: hidden;
    background-color: #1a1a1a;
    border: 1px solid #ffffff20;
    border-radius: 8px;
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
    margin-bottom: 10px;
    box-sizing: border-box;
    border: none;
    border-bottom: 1px solid #ffffff20;
    outline: none;
    background: none;
    height: 50px;
  }
  .results {
    margin: 0;
    padding: 0;
    flex: 1;
    overflow-y: auto;
    box-sizing: border-box;
  }
</style>
