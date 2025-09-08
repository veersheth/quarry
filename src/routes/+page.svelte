<script lang="ts">
  import ResultsList from "../lib/ResultsList.svelte";
  // import ResultsGrid from "../lib/ResultsGrid.svelte";

  let query: string = "";
  let type: string = "apps";
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
      <ResultsList {query} {type}/>
    </div>
  </div>
</main>

<style>
  .container {
    background-color: #dbdbdb;
    display: flex;
    flex: 1;
    height: 95vh;
    flex-direction: column;
    margin: 0;
    padding: 0;
    overflow: none;
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
    padding: 10px;
    margin: 4px auto;
    box-sizing: border-box; 
  }

  .results {
    margin: 0;
    padding: 0;
    flex: 1;
    overflow-y: auto;
    box-sizing: border-box;
  }
</style>
