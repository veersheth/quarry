<!-- lib/Toast.svelte -->
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

