<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { iconSrc, strHue } from "../utils";

  export let icon: string;
  export let name: string = "";
  /** If set, the icon becomes a drag handle for native file drag. */
  export let draggable_path: string | undefined = undefined;

  $: src = iconSrc(icon);

  let loaded = false;
  let error = false;

  function nameInitial(n: string): string {
    return n.trim().charAt(0).toUpperCase() || "?";
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="wrap"
  class:drag-source={!!draggable_path}
  on:mousedown|preventDefault={(e) => {
    if (!draggable_path || e.button !== 0) return;
    invoke("start_drag", { path: draggable_path }).catch(console.error);
  }}
  on:click={(e) => { if (draggable_path) e.stopPropagation(); }}
>
  {#if error}
    <div class="avatar" style="--hue: {strHue(name)}">
      {nameInitial(name)}
    </div>
  {/if}
  <img
    class="icon"
    class:loaded
    {src}
    alt=""
    draggable="false"
    on:load={() => { loaded = true; }}
    on:error={() => { error = true; }}
  />
</div>

<style>
  .wrap {
    position: relative;
    width: 20px;
    height: 20px;
    flex-shrink: 0;
  }

  .drag-source         { cursor: grab; }
  .drag-source:active  { cursor: grabbing; }

  .avatar {
    position: absolute;
    inset: 0;
    border-radius: 5px;
    background: hsl(var(--hue), 35%, 22%);
    color: hsl(var(--hue), 60%, 68%);
    font-size: 0.65em;
    font-weight: 700;
    display: flex;
    align-items: center;
    justify-content: center;
    letter-spacing: 0;
  }

  .icon {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: contain;
    object-position: center;
    opacity: 0;
    transition: opacity 0.15s ease;
  }

  .icon.loaded { opacity: 1; }
</style>
