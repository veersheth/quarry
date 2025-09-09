<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let { query, type, execute } = $props();

  type ListItem = { name: string; executable: string };

  let listitems: ListItem[] = $state([]);

  async function loadApps() {
    try {
      listitems = await invoke<ListItem[]>("get_apps");
    } catch (e) {
      console.error(e);
      listitems = [
        {
          name: "Couldn't resolve apps from backend",
          executable: "notify-send 'Error'",
        },
      ];
    }
  }

  loadApps();

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

<div class="app-list">
  {#each filteredItems as item}
    <button on:click={() => execute(item.executable)} class="app-item">
      <span class="item-name">{item.name}</span>
      <span class="item-desc">This is a description</span>
    </button>
  {/each}
</div>

<style>
  .app-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 5px 0;
  }

  .app-item {
    display: block;
    width: auto;
    padding: 12px 18px;
    margin: 0 8px;
    border: none;
    border-radius: 12px;
    background: none;
    text-align: left;
    color: #e0e0e0;
    cursor: pointer;
    box-sizing: border-box;
  }

  .app-item:hover,
  .app-item:focus-visible {
    background-color: #a3c6ff10;
    border: none;
    outline: none;
  }

  .item-desc {
    opacity: 0.4;
    margin: auto 0.7rem;
    font-size: 16px;
  }
</style>
