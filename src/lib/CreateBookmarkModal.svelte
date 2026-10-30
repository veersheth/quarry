<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { get } from "svelte/store";
  import { bookmarkBeingEdited, bookmarkCreatorOpen, query } from "../stores/search";
  import { refreshSearchInPlace } from "./searcher";
  import { addToast } from "../stores/toasts";

  let nameInput: HTMLInputElement;
  const bookmark = get(bookmarkBeingEdited);
  let name = bookmark?.name ?? "";
  let url = bookmark?.url ?? "";
  let busy = false;

  onMount(() => nameInput?.focus());

  function close() {
    bookmarkCreatorOpen.set(false);
    bookmarkBeingEdited.set(null);
  }

  async function submit() {
    const cleanName = name.trim();
    const cleanUrl = url.trim();
    if (!cleanName || !cleanUrl || busy) return;

    busy = true;
    try {
      if (bookmark) {
        await invoke("update_bookmark", {
          oldName: bookmark.name,
          name: cleanName,
          url: cleanUrl,
        });
      } else {
        await invoke("create_bookmark", { name: cleanName, url: cleanUrl });
      }
      close();
      await refreshSearchInPlace(get(query));
      addToast(bookmark ? "Bookmark updated" : "Bookmark created");
    } catch (error: any) {
      addToast(error?.message ?? `Could not ${bookmark ? "update" : "create"} bookmark`, "error");
    } finally {
      busy = false;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    event.stopPropagation();
    if (event.key === "Escape") {
      event.preventDefault();
      close();
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="overlay" role="dialog" aria-modal="true" aria-labelledby="bookmark-title" on:click={close} on:keydown={handleKeydown}>
  <section class="card" on:click|stopPropagation>
    <header>
      <h2 id="bookmark-title">{bookmark ? "Edit bookmark" : "Create new bookmark"}</h2>
      <p>Save a website so you can find it from Quarry.</p>
    </header>

    <form on:submit|preventDefault={submit}>
      <label>
        <span>Name</span>
        <input bind:this={nameInput} bind:value={name} placeholder="e.g. Project docs" autocomplete="off" />
      </label>
      <label>
        <span>Website URL</span>
        <input bind:value={url} placeholder="example.com" inputmode="url" autocomplete="url" />
      </label>
      <div class="buttons">
        <button type="button" class="cancel" on:click={close}>Cancel</button>
        <button type="submit" class="save" disabled={!name.trim() || !url.trim() || busy}>
          {busy ? "Saving…" : bookmark ? "Save changes" : "Save bookmark"}
        </button>
      </div>
    </form>
  </section>
</div>

<style>
  .overlay {
    position: absolute;
    inset: 0;
    z-index: 2100;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
    background: rgba(0, 0, 0, 0.62);
  }

  .card {
    width: min(420px, 100%);
    padding: 24px;
    border: 1px solid var(--q-border-strong);
    border-radius: 20px;
    background: var(--q-overlay);
    color: var(--q-font-color, #fff);
    box-shadow: 0 0 40px 8px rgba(0, 0, 0, 0.7);
  }

  h2 {
    margin: 0;
    font-size: 1.05rem;
    font-weight: 600;
  }

  header p {
    margin: 6px 0 20px;
    color: var(--q-text-secondary);
    font-size: 0.88rem;
  }

  form,
  label {
    display: flex;
    flex-direction: column;
  }

  form { gap: 16px; }
  label { gap: 7px; }

  label span {
    color: var(--q-text-secondary);
    font-size: 0.82rem;
  }

  input {
    width: 100%;
    box-sizing: border-box;
    padding: 11px 12px;
    border: 1px solid var(--q-border-medium);
    border-radius: 10px;
    outline: none;
    background: var(--q-surface-subtle);
    color: var(--q-font-color, #fff);
    font: inherit;
  }

  input:focus { border-color: var(--q-border-strong); }

  .buttons {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 4px;
  }

  button {
    padding: 9px 14px;
    border: 1px solid var(--q-border-medium);
    border-radius: 10px;
    background: var(--q-surface-subtle);
    color: var(--q-font-color, #fff);
    font: inherit;
  }

  .save { border-color: var(--q-border-strong); }
  button:disabled { opacity: 0.45; }
</style>
