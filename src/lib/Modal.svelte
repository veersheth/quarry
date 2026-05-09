<script lang="ts">
  import { scale } from "svelte/transition";
  import { cubicOut, cubicIn } from "svelte/easing";
  import { marked } from "marked";
  import { invoke } from "@tauri-apps/api/core";
  import { modalStore, closeModal } from "../stores/search";
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="overlay" role="dialog" aria-modal="true" on:click={closeModal}>
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div
    class="card"
    on:click|stopPropagation
    in:scale={{ duration: 160, start: 0.88, opacity: 0, easing: cubicOut }}
    out:scale={{ duration: 110, start: 0.94, opacity: 0, easing: cubicIn }}
  >
    <div class="body">
      {@html marked.parse($modalStore.body)}
    </div>

    <div class="buttons" class:stacked={$modalStore.buttons.length > 2}>
      {#each $modalStore.buttons as btn, i}
        {@const isActive = i === $modalStore.activeIndex}
        <button
          class="btn"
          class:active={isActive}
          class:danger={btn.kind === 'danger'}
          class:primary={btn.kind === 'primary'}
          on:click={async () => {
            if (btn.shell) await invoke("exec_shell", { command: btn.shell });
            closeModal();
          }}
          on:mouseenter={() => modalStore.update(s => ({ ...s, activeIndex: i }))}
        >
          <span class="btn-label">{btn.label}</span>
          {#if isActive}
            <span class="btn-hint">↵</span>
          {/if}
        </button>
      {/each}
    </div>

    <p class="nav-hint">tab to navigate · esc to dismiss</p>
  </div>
</div>

<style>
  .overlay {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2100;
  }

  .card {
    background: var(--q-overlay);
    border: 2px solid var(--q-border-strong);
    border-radius: 20px;
    padding: 24px 24px 16px;
    max-width: 360px;
    width: 90%;
    display: flex;
    flex-direction: column;
    gap: 20px;
    box-shadow: 0 0 40px 8px rgba(0, 0, 0, 0.9);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
  }

  .body {
    color: var(--q-font-color, #fff);
    font-size: 0.95em;
    line-height: 1.6;
  }

  .body :global(p) { margin: 0 0 0.5em; }
  .body :global(p:last-child) { margin-bottom: 0; }
  .body :global(strong) { color: #fff; }

  .buttons {
    display: flex;
    gap: 8px;
  }

  .buttons.stacked {
    flex-direction: column;
  }

  .btn {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 11px 16px;
    border-radius: 12px;
    border: 2px solid transparent;
    background: var(--q-surface-subtle);
    color: var(--q-text-secondary);
    cursor: pointer;
    font-size: 0.9em;
    font-family: var(--q-sans);
    transition: all 0.1s ease;
  }

  .btn.active {
    transform: scale(1.01);
    background: var(--q-active-bg-color);
    border-color: var(--q-active-border-color);
    color: var(--q-font-color, #fff);
    box-shadow: 0 0 10px 1px var(--q-glow);
  }

  .btn.danger {
    border-color: rgba(248, 113, 113, 0);
    color: rgba(248, 113, 113, 0.7);
  }

  .btn.danger.active {
    background: rgba(248, 113, 113, 0.12);
    border-color: rgba(248, 113, 113, 0.7);
    color: #f87171;
    box-shadow: 0 0 10px 1px rgba(248, 113, 113, 0.2);
  }

  .btn.primary {
    border-color: rgba(96, 165, 250, 0);
    color: rgba(96, 165, 250, 0.7);
  }

  .btn.primary.active {
    background: rgba(96, 165, 250, 0.12);
    border-color: rgba(96, 165, 250, 0.7);
    color: #60a5fa;
    box-shadow: 0 0 10px 1px rgba(96, 165, 250, 0.2);
  }

  .btn-label {
    flex: 1;
    text-align: left;
  }

  .btn-hint {
    font-size: 0.85em;
    opacity: 0.5;
    flex-shrink: 0;
  }

  .nav-hint {
    margin: 0;
    font-size: 0.68em;
    letter-spacing: 0.05em;
    opacity: 0.2;
    text-align: center;
    color: var(--q-font-color, #fff);
  }
</style>
