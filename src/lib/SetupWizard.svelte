<script lang="ts">
  import { createEventDispatcher, tick } from "svelte";
  import { fly } from "svelte/transition";
  import { addToast } from "../stores/toasts";

  export let clipboardEnabled = false;
  export let fileIndexingEnabled = false;

  const dispatch = createEventDispatcher<{
    complete: { clipboardEnabled: boolean; fileIndexingEnabled: boolean };
  }>();

  const totalSteps = 5;
  const reducedMotion =
    typeof window !== "undefined" && window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  let step = 0;
  let direction = 1;
  let clipboardPhrase = clipboardEnabled ? "enable" : "";
  let fileIndexingPhrase = fileIndexingEnabled ? "enable" : "";
  let clipboardInput: HTMLInputElement;
  let fileIndexingInput: HTMLInputElement;

  $: if (step === 2) tick().then(() => clipboardInput?.focus());
  $: if (step === 3) tick().then(() => fileIndexingInput?.focus());

  function updateClipboard(event: Event) {
    const value = (event.currentTarget as HTMLInputElement).value;
    const enabled = value.trim().toLowerCase() === "enable";
    if (enabled === clipboardEnabled) return;
    clipboardEnabled = enabled;
    addToast(`Clipboard tracking ${enabled ? "enabled" : "disabled"}`);
  }

  function updateFileIndexing(event: Event) {
    const value = (event.currentTarget as HTMLInputElement).value;
    const enabled = value.trim().toLowerCase() === "enable";
    if (enabled === fileIndexingEnabled) return;
    fileIndexingEnabled = enabled;
    addToast(`File indexing ${enabled ? "enabled" : "disabled"}`);
  }

  function finish() {
    dispatch("complete", { clipboardEnabled, fileIndexingEnabled });
  }

  function next() {
    direction = 1;
    if (step === totalSteps - 1) finish();
    else step += 1;
  }

  function previous() {
    direction = -1;
    step = Math.max(0, step - 1);
  }

  function handleInputKeydown(event: KeyboardEvent) {
    event.stopPropagation();
    if (event.key === "Enter") {
      event.preventDefault();
      next();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    const target = event.target;
    const isTextInput = target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement;

    if (event.key === "ArrowLeft" && !isTextInput) {
      event.preventDefault();
      previous();
    } else if (event.key === "ArrowRight" && !isTextInput) {
      event.preventDefault();
      next();
    } else if (event.key === "Enter") {
      event.preventDefault();
      next();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<section class="setup" aria-label="Quarry onboarding">
  <button class="skip" on:click={finish}>Skip</button>

  <div class="content" aria-live="polite">
    {#key step}
      <div
        class="step"
        in:fly={{ x: direction * 28, duration: reducedMotion ? 0 : 180, opacity: 0 }}
      >
        {#if step === 0}
          <div class="welcome">
            <h1>quarry</h1>
            <p>a general purpose launcher interface</p>
          </div>
        {:else if step === 1}
          <h2>Search with a prefix</h2>

          <table class="prefix-table">
            <tbody>
              <tr><th scope="row"><code>app</code></th><td>Apps</td></tr>
              <tr><th scope="row"><code>f</code></th><td>Files</td></tr>
              <tr><th scope="row"><code>cp</code></th><td>Clipboard</td></tr>
              <tr><th scope="row"><code>em</code></th><td>Emoji</td></tr>
              <tr><th scope="row"><code>def</code></th><td>Definitions</td></tr>
              <tr><th scope="row"><code>=</code></th><td>Calculator</td></tr>
              <tr><th scope="row"><code>space</code></th><td>Prefix a query with a space to ask AI</td></tr>
            </tbody>
          </table>

          <p>Use <kbd>↑</kbd> <kbd>↓</kbd> to move and <kbd>Enter</kbd> to open.</p>
          <br>
          <p><kbd>Ctrl</kbd> + <kbd>K</kbd> opens further actions for the selected result.</p>
        {:else if step === 2}
          <h2>Turn on clipboard tracking?</h2>
          <p class="intro">Clipboard history is stored locally.</p>
          <label class="enable-row">
            <span>Type <code>enable</code> to enable</span>
            <input
              aria-label="Type enable to turn on clipboard tracking"
              bind:this={clipboardInput}
              autocomplete="off"
              bind:value={clipboardPhrase}
              placeholder="enable"
              spellcheck="false"
              on:input={updateClipboard}
              on:keydown={handleInputKeydown}
            />
          </label>
        {:else if step === 3}
          <h2>Turn on file indexing?</h2>
          <p class="intro">The file index is stored locally.</p>
          <label class="enable-row">
            <span>Type <code>enable</code> to enable</span>
            <input
              aria-label="Type enable to turn on file indexing"
              bind:this={fileIndexingInput}
              autocomplete="off"
              bind:value={fileIndexingPhrase}
              placeholder="enable"
              spellcheck="false"
              on:input={updateFileIndexing}
              on:keydown={handleInputKeydown}
            />
          </label>
          <p class="hint">Press <kbd>Enter</kbd> to continue.</p>
        {:else}
          <div class="welcome">
            <h1>You're all set!</h1>
          </div>
        {/if}
      </div>
    {/key}
  </div>

  <nav class="navigation" aria-label="Onboarding steps">
    <button class="chevron" aria-label="Previous step" title="Previous step" disabled={step === 0} on:click={previous}>
      <svg viewBox="0 0 20 20" aria-hidden="true"><path d="m12.5 4.5-5 5.5 5 5.5" /></svg>
    </button>
    <button class="chevron" aria-label={step === totalSteps - 1 ? "Finish onboarding" : "Next step"} title={step === totalSteps - 1 ? "Finish" : "Next step"} on:click={next}>
      <svg viewBox="0 0 20 20" aria-hidden="true"><path d="m7.5 4.5 5 5.5-5 5.5" /></svg>
    </button>
  </nav>
</section>

<style>
  .setup {
    display: flex;
    flex: 1;
    min-height: 0;
    flex-direction: column;
    box-sizing: border-box;
    padding: 32px 44px 34px;
    color: var(--q-font-color, #f2f2f2);
    background: var(--q-bg-color, #111113);
    font-family: var(--q-sans);
  }

  .skip {
    align-self: flex-end;
    padding: 5px 7px;
    border: 0;
    background: transparent;
    color: var(--q-text-muted, rgba(255, 255, 255, 0.55));
    font: inherit;
    font-size: 0.86em;
    cursor: pointer;
  }

  .skip:hover { color: var(--q-font-color, #fff); }

  .content {
    display: grid;
    flex: 1;
    min-height: 0;
    align-items: center;
    overflow-y: auto;
  }

  .step {
    grid-area: 1 / 1;
    width: min(620px, 100%);
    justify-self: start;
    margin: 0;
    padding: 28px 0;
  }

  .welcome h1 {
    margin: 0 0 20px;
    font-size: clamp(2.3rem, 6vw, 3.4rem);
    font-weight: 560;
    font-style: italic;
    letter-spacing: -0.045em;
    line-height: 1;
  }

  .welcome p, .intro {
    margin: 0;
    color: var(--q-text-secondary, rgba(255, 255, 255, 0.68));
    font-size: 1.05em;
    line-height: 1.55;
  }

  h2 {
    margin: 0 0 8px;
    font-size: 1.35em;
    font-weight: 560;
    letter-spacing: -0.015em;
  }

  .prefix-table {
    width: 100%;
    margin: 20px 12px;
    border: 1px solid rgba(255, 255, 255, 0.11);
    border-radius: 9px;
    border-spacing: 0;
    overflow: hidden;
    text-align: left;
  }

  .prefix-table th, .prefix-table td {
    padding: 9px 13px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.07);
    font-weight: 400;
    line-height: 1.35;
  }

  .prefix-table tr:last-child th, .prefix-table tr:last-child td { border-bottom: 0; }
  .prefix-table th { width: 78px; }
  .prefix-table code, .enable-row input { font-family: var(--q-mono); }
  .prefix-table code { color: var(--q-font-color, #eee); }
  .prefix-table td, .enable-row > span, .hint {
    color: var(--q-text-muted, rgba(255, 255, 255, 0.55));
    font-size: 0.9em;
  }

  kbd {
    padding: 2px 5px;
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.045);
    color: var(--q-text-secondary, rgba(255, 255, 255, 0.76));
    font-family: var(--q-mono);
    font-size: 0.86em;
  }

  .hint { margin: 18px 0 0; }

  .enable-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 18px;
    padding: 17px 0;
    margin-top: 20px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.075);
    cursor: text;
  }

  .enable-row input {
    width: 112px;
    padding: 8px 9px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 6px;
    outline: none;
    background: rgba(255, 255, 255, 0.035);
    color: var(--q-font-color, #f2f2f2);
    font-size: 0.86em;
  }

  .enable-row input:focus { border-color: var(--q-active-border-color, rgba(255, 255, 255, 0.4)); }
  .enable-row input::placeholder { color: var(--q-text-dim, rgba(255, 255, 255, 0.38)); }

  .navigation {
    display: flex;
    justify-content: space-between;
    padding-top: 12px;
  }

  .chevron {
    display: grid;
    width: 36px;
    height: 36px;
    place-items: center;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 7px;
    background: transparent;
    color: var(--q-text-secondary, rgba(255, 255, 255, 0.76));
    cursor: pointer;
  }

  .welcome h1 {
    margin: 0 0 20px;
    font-size: clamp(2.3rem, 6vw, 3.4rem);
    font-weight: 560;
    font-style: italic;
    letter-spacing: -0.045em;
    line-height: 1;
  }

  .chevron:hover:not(:disabled) { background: rgba(255, 255, 255, 0.07); }
  .chevron:disabled { opacity: 0.3; cursor: default; }
  .chevron:focus-visible, .skip:focus-visible { outline: 2px solid var(--q-active-border-color, #8ab4f8); outline-offset: 2px; }
  .chevron svg { width: 18px; height: 18px; fill: none; stroke: currentColor; stroke-linecap: round; stroke-linejoin: round; stroke-width: 1.7; }

  @media (max-width: 600px) {
    .setup { padding: 28px 24px 30px; }
    .prefix-table th { width: 58px; }
    .prefix-table th, .prefix-table td { padding: 8px 10px; }
  }
</style>
