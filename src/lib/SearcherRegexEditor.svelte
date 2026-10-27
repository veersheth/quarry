<script lang="ts">
  import { onMount, tick } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { get } from "svelte/store";
  import { addToast } from "../stores/toasts";
  import { activeIndex, query, resultItems, resultType, searcherName } from "../stores/search";
  import { search } from "./searcher";

  type EditorRequest = {
    config_key: string;
    name: string;
    mode: "prefix" | "regex";
    value: string;
  };

  let request: EditorRequest | null = null;
  let input: HTMLInputElement;
  let saving = false;

  onMount(() => {
    let unlisten: (() => void) | undefined;
    listen<EditorRequest>("quarry-edit-searcher-trigger", async ({ payload }) => {
      request = payload;
      saving = false;
      await tick();
      input?.focus();
      input?.select();
    }).then((stop) => (unlisten = stop));

    return () => unlisten?.();
  });

  function close() {
    request = null;
    window.dispatchEvent(new Event("quarry-overlay-closed"));
  }

  async function save() {
    if (!request || saving) return;
    saving = true;
    try {
      await invoke("save_searcher_trigger", {
        configKey: request.config_key,
        mode: request.mode,
        value: request.value,
      });
      const refreshed = await search(get(query));
      if (refreshed) {
        resultItems.set(refreshed.results);
        resultType.set(refreshed.result_type);
        searcherName.set(refreshed.searcher ?? "");
        activeIndex.set(0);
      }
      addToast(`${request.name} ${request.mode === "prefix" ? "prefix" : "trigger regex"} saved`);
      close();
    } catch (error) {
      addToast(String(error), "error");
    } finally {
      saving = false;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    event.stopPropagation();
    if (event.key === "Escape") {
      event.preventDefault();
      close();
    } else if (event.key === "Enter") {
      event.preventDefault();
      void save();
    }
  }
</script>

{#if request}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="regex-overlay" role="presentation" on:click={close}>
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <section
      class="regex-dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="regex-title"
      on:click|stopPropagation
    >
      <h2 id="regex-title">{request.name} {request.mode === "prefix" ? "prefix" : "trigger regex pattern"}</h2>
      <p>
        {request.mode === "prefix"
          ? "Enter a single prefix. Quarry will create the matching trigger regex."
          : "Enter the regex Quarry uses to recognize this searcher."}
      </p>
      <label for="searcher-regex">{request.mode === "prefix" ? "Prefix" : "Trigger regex pattern"}</label>
      <input
        id="searcher-regex"
        bind:this={input}
        bind:value={request.value}
        spellcheck="false"
        on:keydown={handleKeydown}
      />
      <div class="regex-actions">
        <span>Enter to save · Esc to cancel</span>
        <button on:click={save} disabled={saving}>{saving ? "Saving…" : "Save"}</button>
      </div>
    </section>
  </div>
{/if}

<style>
  .regex-overlay {
    position: fixed;
    inset: 0;
    z-index: 2200;
    display: grid;
    place-items: center;
    background: rgba(0, 0, 0, 0.58);
  }

  .regex-dialog {
    width: min(520px, calc(100vw - 40px));
    padding: 22px;
    border: 1px solid var(--q-border-strong);
    border-radius: 16px;
    background: var(--q-overlay);
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.55);
    color: var(--q-font-color, #fff);
  }

  h2 {
    margin: 0 0 8px;
    font-size: 1rem;
    font-weight: 600;
  }

  p {
    margin: 0 0 16px;
    color: var(--q-text-secondary, rgba(255, 255, 255, 0.62));
    font-size: 0.86rem;
  }

  label {
    display: block;
    margin-bottom: 6px;
    color: var(--q-text-secondary, rgba(255, 255, 255, 0.72));
    font-size: 0.8rem;
  }

  input {
    box-sizing: border-box;
    width: 100%;
    padding: 11px 12px;
    border: 1px solid var(--q-border-medium);
    border-radius: 9px;
    outline: none;
    background: var(--q-code-bg, rgba(0, 0, 0, 0.25));
    color: var(--q-font-color, #fff);
    font: 0.9rem var(--q-mono, monospace);
  }

  input:focus {
    border-color: var(--q-border-strong);
  }

  .regex-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-top: 16px;
    color: var(--q-text-secondary, rgba(255, 255, 255, 0.5));
    font-size: 0.76rem;
  }

  button {
    padding: 7px 14px;
    border: 1px solid var(--q-border-medium);
    border-radius: 8px;
    background: var(--q-surface-subtle);
    color: var(--q-font-color, #fff);
    font: inherit;
    cursor: pointer;
  }
</style>
