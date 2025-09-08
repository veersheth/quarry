<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  export let query;

  type AppInfo = {
    name: string;
    executable: string;
  };

  let apps: AppInfo[] = [];
  let error: string | null = null;

  async function loadApps() {
    try {
      const res = await invoke<AppInfo[]>("get_apps");
      apps = res;
    } catch (e) {
      console.error(e);
      error = "Failed to fetch apps from Tauri backend.";
    }
  }

  loadApps();

  async function handleClick(app: AppInfo) {
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
    {#each apps as app}
      <div on:click={() => handleClick(app)} class="app-item">
        {app.name}
      </div>
    {/each}
  </ul>
{/if}

<style>
  ul {
    /* background-color: red; */
  }
</style>
