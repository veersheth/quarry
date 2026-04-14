<script lang="ts">
  import { toasts } from "../stores/toasts";
  import { fly } from "svelte/transition";
</script>

<div class="toast-container">
  {#each $toasts as toast (toast.id)}
    <div
      class="toast {toast.type}"
      in:fly={{ y: 20, duration: 200 }}
      out:fly={{ y: 20, duration: 150 }}
    >
      {#if toast.type === "success"}✓{:else if toast.type === "error"}✕{:else}·{/if}
      {toast.message}
    </div>
  {/each}
</div>

<style>
  .toast-container {
    position: fixed;
    bottom: 16px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    flex-direction: column;
    gap: 8px;
    align-items: center;
    pointer-events: none;
    z-index: 1000;
  }

  .toast {
    padding: 8px 16px;
    border-radius: 20px;
    font-size: 0.8rem;
    font-family: "JetBrainsMono Nerd Font", monospace;
    border: 1px solid;
    backdrop-filter: blur(8px);
    white-space: nowrap;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .success {
    background: rgba(34, 197, 94, 0.15);
    border-color: rgba(34, 197, 94, 0.4);
    color: #86efac;
  }

  .error {
    background: rgba(239, 68, 68, 0.15);
    border-color: rgba(239, 68, 68, 0.4);
    color: #fca5a5;
  }

  .info {
    background: rgba(59, 130, 246, 0.15);
    border-color: rgba(59, 130, 246, 0.4);
    color: #93c5fd;
  }
</style>
