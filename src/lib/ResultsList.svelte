<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  let {query, type, execute} = $props();

  type ListItem = {
    name: string;
    executable: string;
  };

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
</script>

{#if error}
  <div class="error">{error}</div>
{:else}
    {#each listitems as item}
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
    background-color: #EFEFEF;
  }

  .app-item:hover {
    background-color: #ffffff;
  }
</style>
