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
    --q-sans: var(--q-font-family, "Inter", "Segoe UI", "Adwaita Sans", "Noto Color Emoji", sans-serif);
    --q-mono: var(--q-monospace-font-family, "JetBrainsMono Nerd Font", "Fira Code", "Cascadia Mono", monospace);
  }

  .root {
    opacity: 0;
    transition: opacity 0.08s ease;
  }

  .root.mounted {
    opacity: 1;
  }
</style>
