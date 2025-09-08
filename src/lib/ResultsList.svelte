<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  export let query;

  type ListItem = {
    name: string;
    executable: string;
  };

  let listitems: ListItem[] = [];
  let error: string | null = null;

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
    try {
      await invoke("run_app", { executable: app.executable });
      console.log(`Launched ${app.name}`);
    } catch (e) {
      alert(`Failed to launch ${app.name}: ${e}`);
    }

  }
</script>

{#if error}
  <div class="error">{error}</div>
{:else}
  <ul>
    {#each listitems as item}
      <div on:click={() => handleClick(item)} class="app-item">
        {item.name}
      </div>
    {/each}
  </ul>
{/if}

<style>
  .app-item {
    cursor: pointer;
  }

  .app-item:hover {
    background-color: #ffffff;
  }
</style>
