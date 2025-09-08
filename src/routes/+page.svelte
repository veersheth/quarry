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
      <ResultsList />
    </div>
  </div>
</main>

<style>
  .container {
    background-color: #dbdbdb;
    display: flex;
    flex-direction: column;
    height: 95vh;
    margin: 0;
    padding: 0;
  }

  .search {
    display: block;
    width: 100%;
    padding: 1rem;
    box-sizing: border-box; 
  }

  .results {
    display: block;
    width: 100%;
    padding: 1rem;
    box-sizing: border-box; 
  }
</style>
