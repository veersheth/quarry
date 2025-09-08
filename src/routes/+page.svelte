<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import ResultsList from "../lib/ResultsList.svelte";
  // import ResultsGrid from "../lib/ResultsGrid.svelte";

  let query: string = "";
  let type: string = "apps";

  async function execute(executable: String) {
    try {
      const res = await invoke<ListItem[]>("execute", {
        executable: executable,
      });
    } catch (e) {
      console.error(e);
    }
  }

  let view: "list" | "grid" = "list";
</script>

<main class="container">
  <div class="panel">
    <input
      type="text"
      placeholder="Search…"
      bind:value={query}
      class="search"
    />

    <div class="results">
      <ResultsList {query} {type} {execute} />
    </div>
  </div>
</main>

<style>
  .container {
    display: flex;
    flex: 1;
    height: 95vh;
    flex-direction: column;
    margin: 0;
    padding: 0;
    overflow: none;

    * {
      background-color: #1a1a1a;
      color: #ffffff;
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
    border-bottom: solid 1px white;
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
