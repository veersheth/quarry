<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { marked } from "marked";

  interface ModalButton {
    label: string;
    kind?: string;
    shell?: string;
  }

  export let body: string;
  export let buttons: ModalButton[] = [{ label: "Dismiss" }];
  export let onClose: () => void;
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div
  class="overlay"
  role="dialog"
  aria-modal="true"
  tabindex="-1"
  on:click={onClose}
  on:keydown={(e) => { if (e.key === "Escape") onClose(); }}
>
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="card" on:click|stopPropagation on:keydown|stopPropagation>
    <div class="body">
      {@html marked.parse(body)}
    </div>
    <div class="buttons">
      {#each buttons as btn}
        <button
          class="btn {btn.kind ?? ''}"
          on:click={async () => {
            if (btn.shell) await invoke("exec_shell", { command: btn.shell });
            onClose();
          }}
        >{btn.label}</button>
      {/each}
    </div>
  </div>
</div>

<style>
  .overlay {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.65);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2100;
    backdrop-filter: blur(4px);
  }

  .card {
    background: var(--q-bg-color, rgba(20, 20, 20, 1));
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 14px;
    padding: 8px 32px 20px;
    max-width: 380px;
    width: 90%;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .body {
    color: var(--q-font-color, #fff);
  }

  .buttons {
    display: flex;
    gap: 8px;
    justify-content: space-between;
  }

  .btn {
    flex: 1;
    padding: 12px 18px;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.2);
    background: rgba(255, 255, 255, 0.08);
    color: var(--q-font-color, #fff);
    cursor: pointer;
    font-size: 13px;
    transition: background 0.15s;
  }

  .btn:hover {
    background: rgba(255, 255, 255, 0.15);
  }

  .btn.danger {
    border-color: rgba(248, 113, 113, 0.4);
    color: #f87171;
  }

  .btn.danger:hover {
    background: rgba(248, 113, 113, 0.15);
  }

  .btn.primary {
    border-color: rgba(96, 165, 250, 0.4);
    color: #60a5fa;
  }

  .btn.primary:hover {
    background: rgba(96, 165, 250, 0.15);
  }
</style>
