<script lang="ts">
  import { onMount } from "svelte";

  let mounted = false;

  onMount(() => {
    mounted = true;
  });
</script>

<div class="root" class:mounted>
  <slot />
</div>

<style>
  :global(:root) {
    --q-sans: "Inter", "Segoe UI", "Adwaita Sans", "Noto Color Emoji", sans-serif;
    --q-mono: "JetBrainsMono Nerd Font", "Fira Code", "Cascadia Mono", monospace;
  }

  .root {
    opacity: 0;
    transition: opacity 0.08s ease;
  }

  .root.mounted {
    opacity: 1;
  }
</style>
