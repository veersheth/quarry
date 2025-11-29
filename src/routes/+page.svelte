<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import ResultsList from "../lib/ResultsList.svelte";

  let query: string = "";
  let type: string = "apps";

  type ListItem = {
    name: string;
    exec: string;
    description?: string;
    icon?: string;
  };

  let listitems: ListItem[] = [];

  async function execute(executable: string) {
    try {
      await invoke("execute", { executable });
    } catch (e) {
      console.error(e);
    }
  }

  async function loadApps() {
    try {
      listitems = await invoke<ListItem[]>("get_apps");
    } catch (e) {
      console.error(e);
      listitems = [
        {
          name: "Couldn't resolve apps from backend",
          exec: "notify-send 'Error'",
        },
      ];
    }
  }

  loadApps();

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
      <ResultsList {listitems} {query} {execute} />
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
    border: 2px solid #ffffff20;
    border-radius: 8px;
    * {
      color: #fffffff8;
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
