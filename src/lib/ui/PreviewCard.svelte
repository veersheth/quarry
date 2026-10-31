<script lang="ts">
  import ItemIcon from "./ItemIcon.svelte";
  import { iconSrc } from "../utils";

  export let icon: string | null | undefined = undefined;
  export let title = "";
  export let subtitle: string | undefined = undefined;
  export let wrapTitle = false;
</script>

<div class="preview-card-container">
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

    <header class="preview-heading" class:wrap-title={wrapTitle}>
      {#if icon}
        <ItemIcon {icon} name={title} size={34} />
      {:else}
        <div class="fallback-icon">{title.trim().charAt(0).toUpperCase() || "?"}</div>
      {/if}
      <div class="heading-copy">
        <h2 class:wrap-title={wrapTitle}>{title}</h2>
        {#if subtitle}<p>{subtitle}</p>{/if}
      </div>
    </header>

    <div class="preview-content"><slot /></div>
  </article>
</div>

<style>
  .preview-card-container {
    width: 100%;
    min-width: 0;
    container: preview-card / inline-size;
  }

  .preview-card {
    position: relative;
    isolation: isolate;
    display: flex;
    width: 100%;
    max-height: 100%;
    box-sizing: border-box;
    flex-direction: column;
    gap: 14px;
    overflow: auto;
    margin: 0;
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

  .heading-copy {
    display: flex;
    min-width: 0;
    flex: 1;
    flex-direction: column;
    gap: 8px;
  }

  .preview-heading.wrap-title { align-items: flex-start; }

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

  h2.wrap-title {
    overflow: visible;
    overflow-wrap: anywhere;
    text-overflow: clip;
    white-space: normal;
  }

  p {
    overflow: hidden;
    margin: 0;
    color: var(--q-text-dim, rgba(255, 255, 255, 0.62));
    font-size: 0.78em;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .preview-content { min-width: 0; }

  @container preview-card (max-width: 460px) {
    .preview-card {
      gap: 12px;
      padding: 14px;
    }

    .preview-heading { gap: 9px; }

    h2:not(.wrap-title) {
      display: -webkit-box;
      overflow: hidden;
      white-space: normal;
      overflow-wrap: anywhere;
      -webkit-box-orient: vertical;
      -webkit-line-clamp: 2;
    }

    :global(.metadata-table th) {
      width: 76px;
      padding-inline: 8px;
    }

    :global(.metadata-table td) { padding-inline: 8px; }
  }
</style>
