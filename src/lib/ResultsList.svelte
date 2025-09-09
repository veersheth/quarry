<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let { query, type, execute } = $props();

  type ListItem = {
    name: string;
    exec: string;
    description?: string;
    icon?: string;
  };

  let listitems: ListItem[] = $state([]);

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

  function truncate(str: string | undefined, maxLength: number): string {
    if (!str) return "";
    return str.length > maxLength ? str.slice(0, maxLength) + "…" : str;
  }
</script>

<div class="app-list">
  {#each filteredItems as item}
    <!-- <button onclick={() => execute(item.exec)} class="app-item"> -->
    <button onclick={() => alert(item.icon)} class="app-item">
        <img class="item-icon" src={item.icon} alt=""/>
      <span class="item-name">{item.name}</span>
      <span class="item-desc">{truncate(item.description, 70)}</span>
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
    display: flex;
    width: auto;
    padding: 12px 18px;
    margin: 0 8px;
    border: none;
    border-radius: 12px;
    background: none;
    text-align: left;
    color: #e0e0e0;
    cursor: pointer;
  }

  .app-item:hover,
  .app-item:focus-visible {
    background-color: #a3c6ff10;
    border: none;
    outline: none;
  }

  .item-icon {
    width: 20px;
    height: 20px;
  }

  .item-name {
    margin: auto 0.7rem;
  }

  .item-desc {
    opacity: 0.4;
    font-size: 16px;
  }
</style>
