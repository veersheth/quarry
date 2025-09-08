<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let { query, type, execute } = $props();

  type ListItem = { name: string; executable: string };

  let listitems: ListItem[] = $state([]);

  let error: string | null = $state(null);

  async function loadApps() {
    try {
      const res = await invoke<ListItem[]>("get_apps");
      listitems = res;
    } catch (e) {
      console.error(e);
      error = "Failed to fetch apps from Tauri backend.";
    }
  }

  loadApps();

  async function handleClick(app: ListItem) {
    execute(app.executable);
  }

  function fuzzySearch(str: string, query: string): boolean {
    str = str.toLowerCase();
    query = query.toLowerCase();
    let i = 0,
      lastSearched = -1,
      current = query[i];
    while (current) {
      lastSearched = str.indexOf(current, lastSearched + 1);
      if (lastSearched === -1) return false;
      current = query[++i];
    }
    return true;
  }

  let filteredItems = $derived(
    listitems.filter((item) => fuzzySearch(item.name, query)),
  );
</script>

{#if error}
  <div class="error">{error}</div>
{:else}
  {#each filteredItems as item}
    <div on:click={() => handleClick(item)} class="app-item">
      {item.name}
    </div>
  {/each}
{/if}

<style>
  .app-item {
    padding: 8px 14px;
    margin: 0;
    cursor: pointer;
    color: black;
  }

  .app-item:hover {
    background-color: #ffffff;
  }
</style>
