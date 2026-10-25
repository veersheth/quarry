<script lang="ts">
  import ItemIcon from "./ItemIcon.svelte";
  import { iconSrc } from "../utils";

  export let icon: string | null | undefined = undefined;
  export let title = "";
  export let subtitle: string | undefined = undefined;
</script>

<article class="preview-card">
  {#if icon}
    <img
      class="icon-glow"
      src={iconSrc(icon)}
      alt=""
      aria-hidden="true"
      on:error={(event) => ((event.currentTarget as HTMLImageElement).style.display = "none")}
    />
  {/if}

  <header class="preview-heading">
    {#if icon}
      <ItemIcon {icon} name={title} size={34} />
    {:else}
      <div class="fallback-icon">{title.trim().charAt(0).toUpperCase() || "?"}</div>
    {/if}
    <div class="heading-copy">
      <h2>{title}</h2>
      {#if subtitle}<p>{subtitle}</p>{/if}
    </div>
  </header>

  <div class="preview-content"><slot /></div>
</article>

<style>
  .preview-card {
    position: relative;
    isolation: isolate;
    display: flex;
    width: min(100%, 560px);
    max-height: 100%;
    box-sizing: border-box;
    flex-direction: column;
    gap: 14px;
    overflow: auto;
    margin: auto;
    padding: 18px;
    border: 1px solid var(--q-card-border, rgba(255, 255, 255, 0.09));
    border-radius: var(--q-card-radius, 15px);
    background: var(--q-card-surface, #181818);
    box-shadow:
      0 12px 30px rgba(0, 0, 0, 0.18),
      inset 0 1px 0 rgba(255, 255, 255, 0.045);
  }

  .icon-glow {
    position: absolute;
    z-index: 0;
    top: 12px;
    left: 12px;
    width: 46px;
    height: 46px;
    object-fit: contain;
    filter: blur(17px) saturate(1.35);
    opacity: 0.16;
    mix-blend-mode: screen;
    pointer-events: none;
  }

  .preview-heading,
  .preview-content {
    position: relative;
    z-index: 1;
  }

  .preview-heading {
    display: flex;
    min-width: 0;
    align-items: center;
    gap: 12px;
  }

  .heading-copy { min-width: 0; }

  .fallback-icon {
    display: grid;
    width: 34px;
    height: 34px;
    flex: 0 0 34px;
    place-items: center;
    border: 1px solid var(--q-divider-dark, #2e2e2e);
    border-radius: 8px;
    background: var(--q-surface-dark, #222);
    color: var(--q-text-secondary, #ccc);
    font-size: 0.8em;
  }

  h2 {
    overflow: hidden;
    margin: 0;
    color: var(--q-font-color, #f2f2f2);
    font-size: 1em;
    font-weight: 600;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  p {
    overflow: hidden;
    margin: 3px 0 0;
    color: var(--q-text-dim, rgba(255, 255, 255, 0.62));
    font-size: 0.78em;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .preview-content { min-width: 0; }
</style>
