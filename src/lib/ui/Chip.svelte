<script lang="ts">
  export let href: string | undefined = undefined;
  export let as: "span" | "button" = "span";
  export let mono: boolean = false;
  /** Optional accent color. Derives a matching tinted background and border. */
  export let accent: string | undefined = undefined;

  $: style = accent ? `--accent: ${accent}` : undefined;
</script>

{#if href}
  <a class="chip" class:mono class:accented={!!accent} {href} {style} target="_blank" rel="noopener noreferrer" on:click>
    <slot />
  </a>
{:else if as === "button"}
  <button class="chip" class:mono class:accented={!!accent} {style} on:click>
    <slot />
  </button>
{:else}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <span class="chip" class:mono class:accented={!!accent} {style} on:click>
    <slot />
  </span>
{/if}

<style>
  .chip {
    display: inline-flex;
    align-items: center;
    padding: 2px 10px;
    border-radius: 20px;
    border: 1px solid var(--q-divider-dark);
    color: var(--q-text-dim);
    background: none;
    white-space: nowrap;
    text-decoration: none;
    transition: color 0.12s ease, background 0.12s ease;
  }

  .chip:hover {
    color: var(--q-text-secondary);
    background: var(--q-surface-subtle);
  }

  .chip.mono {
    font-family: var(--q-mono);
  }

  .chip.accented {
    color: var(--accent);
    background: color-mix(in srgb, var(--accent) 10%, transparent);
    border-color: color-mix(in srgb, var(--accent) 25%, transparent);
  }

  .chip.accented:hover {
    background: color-mix(in srgb, var(--accent) 18%, transparent);
  }
</style>
