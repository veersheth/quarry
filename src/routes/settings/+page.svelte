<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { invoke } from "@tauri-apps/api/core";
  import { fly } from "svelte/transition";
  import { backOut } from "svelte/easing";
  import { toasts, addToast } from "../../stores/toasts";

  interface ThemeConfig {
    background_color: string;
    background_opacity: number;
    font_size: number;
    font_color: string;
    border_radius: number;
    border_color: string;
    border_thickness: number;
    item_border_radius: number;
    active_bg_color: string;
    active_border_color: string;
  }

  interface TriggerConfig {
    camera: string; bookmarks: string; files: string; clipboard: string;
    emojis: string; shell: string; lorem: string; math: string;
    dictionary: string; system: string; color_picker: string; apps: string;
    url: string; currency: string; note: string; ai: string;
    time: string; settings: string; windows: string; timer: string;
  }

  interface WebSearchConfig {
    name: string;
    trigger: string;
    url: string;
    icon?: string;
  }

  interface DefaultSearchConfig {
    web_searches: string[];
    max_web_results: number;
  }

  interface Config {
    triggers: TriggerConfig;
    theme: ThemeConfig;
    web_searches: WebSearchConfig[];
    default_search: DefaultSearchConfig;
  }

  let config: Config | null = null;
  let groqApiKey = "";
  let tab: "theme" | "triggers" | "web" | "general" = "theme";
  let saving = false;
  let showApiKey = false;
  let appWindow: ReturnType<typeof getCurrentWindow>;
  let uiScale = 1.0;

  // Convert any color string → hex for the native color picker input
  function toHex(val: string): string {
    if (/^#[0-9a-fA-F]{6}$/.test(val)) return val;
    const m = val.match(/rgba?\(\s*(\d+),\s*(\d+),\s*(\d+)/);
    if (m) return "#" + [m[1], m[2], m[3]]
      .map(n => parseInt(n).toString(16).padStart(2, "0")).join("");
    return "#000000";
  }

  // Convert hex → rgb(...), preserving existing alpha if the field was rgba
  function hexToRgb(hex: string, existing: string): string {
    const r = parseInt(hex.slice(1, 3), 16);
    const g = parseInt(hex.slice(3, 5), 16);
    const b = parseInt(hex.slice(5, 7), 16);
    const a = existing.match(/rgba?\(\s*\d+,\s*\d+,\s*\d+,\s*([\d.]+)/);
    return a ? `rgba(${r},${g},${b},${a[1]})` : `rgb(${r},${g},${b})`;
  }

  function syncColorFromPicker(field: keyof ThemeConfig, hex: string) {
    if (!config) return;
    const existing = (config.theme as any)[field] as string;
    (config.theme as any)[field] = hexToRgb(hex, existing);
    config = config;
  }

  const TRIGGER_LABELS: [keyof TriggerConfig, string][] = [
    ["apps",         "Apps"],
    ["files",        "Files"],
    ["clipboard",    "Clipboard"],
    ["emojis",       "Emojis"],
    ["bookmarks",    "Bookmarks"],
    ["shell",        "Shell"],
    ["math",         "Math"],
    ["dictionary",   "Dictionary"],
    ["color_picker", "Color picker"],
    ["currency",     "Currency"],
    ["time",         "Time"],
    ["ai",           "AI chat"],
    ["note",         "Note"],
    ["timer",        "Timer"],
    ["camera",       "Camera"],
    ["lorem",        "Lorem ipsum"],
    ["system",       "System"],
    ["settings",     "Settings"],
    ["windows",      "Window switcher"],
    ["url",          "URL (auto-detect)"],
  ];

  onMount(async () => {
    appWindow = getCurrentWindow();
    [config, groqApiKey] = await Promise.all([
      invoke<Config>("get_config"),
      invoke<string>("get_groq_api_key"),
    ]);
    document.documentElement.style.setProperty("--q-font-size", `${config.theme.font_size}px`);
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") { e.preventDefault(); appWindow?.hide(); return; }
    if (e.ctrlKey && (e.key === "-" || e.key === "=" || e.key === "+" || e.key === "0")) {
      e.preventDefault();
      if (e.key === "-") uiScale = Math.max(0.6, parseFloat((uiScale - 0.05).toFixed(2)));
      else if (e.key === "=" || e.key === "+") uiScale = Math.min(2.0, parseFloat((uiScale + 0.05).toFixed(2)));
      else uiScale = 1.0;
    }
  }

  async function save() {
    if (!config) return;
    saving = true;
    try {
      await Promise.all([
        invoke("save_config", { config }),
        invoke("save_groq_api_key", { key: groqApiKey }),
      ]);
      addToast("Config saved", "success");
    } catch (err) {
      addToast(String(err), "error", 4000);
    } finally {
      saving = false;
    }
  }

  function addWebSearch() {
    if (!config) return;
    config.web_searches = [...config.web_searches, { name: "", trigger: "", url: "" }];
  }

  function removeWebSearch(i: number) {
    if (!config) return;
    config.web_searches = config.web_searches.filter((_, idx) => idx !== i);
  }

  function toggleDefaultSearch(name: string) {
    if (!config) return;
    const list = config.default_search.web_searches;
    config.default_search.web_searches = list.includes(name)
      ? list.filter(n => n !== name)
      : [...list, name];
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<main style="zoom: {uiScale}">
  <div class="tabs">
    {#each [["theme","Theme"],["triggers","Triggers"],["web","Web searches"],["general","General"]] as [id, label]}
      <button class="tab" class:active={tab === id} on:click={() => (tab = id as any)}>
        {label}
      </button>
    {/each}
  </div>

  <div class="content">
    {#if config === null}
      <div class="loading">Loading…</div>

    {:else if tab === "theme"}

      <div class="group">
        <div class="group-label">Background</div>
        <div class="field">
          <label>Color</label>
          <div class="field-body">
            <div class="swatch" style="background:{config.theme.background_color}">
              <input type="color" value={toHex(config.theme.background_color)}
                on:input={e => syncColorFromPicker("background_color", (e.target as HTMLInputElement).value)} />
            </div>
            <input class="inp" bind:value={config.theme.background_color} placeholder="rgba(10,10,10,1)" />
          </div>
        </div>
        <div class="field">
          <label>Opacity</label>
          <div class="field-body">
            <input type="range" min="0" max="1" step="0.01"
              bind:value={config.theme.background_opacity} class="slider" />
            <span class="mono-val">{config.theme.background_opacity.toFixed(2)}</span>
          </div>
        </div>
      </div>

      <div class="sep" />

      <div class="group">
        <div class="group-label">Typography</div>
        <div class="field">
          <label>Font color</label>
          <div class="field-body">
            <div class="swatch" style="background:{config.theme.font_color}">
              <input type="color" value={toHex(config.theme.font_color)}
                on:input={e => syncColorFromPicker("font_color", (e.target as HTMLInputElement).value)} />
            </div>
            <input class="inp" bind:value={config.theme.font_color} placeholder="rgba(226,219,197,0.88)" />
          </div>
        </div>
        <div class="field">
          <label>Font size</label>
          <div class="field-body">
            <input type="number" min="8" max="32" bind:value={config.theme.font_size}
              class="inp inp-sm" />
            <span class="unit">px</span>
          </div>
        </div>
      </div>

      <div class="sep" />

      <div class="group">
        <div class="group-label">Border</div>
        <div class="field">
          <label>Color</label>
          <div class="field-body">
            <div class="swatch" style="background:{config.theme.border_color}">
              <input type="color" value={toHex(config.theme.border_color)}
                on:input={e => syncColorFromPicker("border_color", (e.target as HTMLInputElement).value)} />
            </div>
            <input class="inp" bind:value={config.theme.border_color} placeholder="rgba(255,255,255,0.35)" />
          </div>
        </div>
        <div class="field">
          <label>Radius</label>
          <div class="field-body">
            <input type="range" min="0" max="40" step="1"
              bind:value={config.theme.border_radius} class="slider" />
            <span class="mono-val">{config.theme.border_radius}</span>
            <span class="unit">px</span>
          </div>
        </div>
        <div class="field">
          <label>Thickness</label>
          <div class="field-body">
            <input type="range" min="0" max="8" step="1"
              bind:value={config.theme.border_thickness} class="slider" />
            <span class="mono-val">{config.theme.border_thickness}</span>
            <span class="unit">px</span>
          </div>
        </div>
      </div>

      <div class="sep" />

      <div class="group">
        <div class="group-label">Active item</div>
        <div class="field">
          <label>Background</label>
          <div class="field-body">
            <div class="swatch" style="background:{config.theme.active_bg_color}">
              <input type="color" value={toHex(config.theme.active_bg_color)}
                on:input={e => syncColorFromPicker("active_bg_color", (e.target as HTMLInputElement).value)} />
            </div>
            <input class="inp" bind:value={config.theme.active_bg_color} placeholder="rgba(40,40,40,1)" />
          </div>
        </div>
        <div class="field">
          <label>Border color</label>
          <div class="field-body">
            <div class="swatch" style="background:{config.theme.active_border_color}">
              <input type="color" value={toHex(config.theme.active_border_color)}
                on:input={e => syncColorFromPicker("active_border_color", (e.target as HTMLInputElement).value)} />
            </div>
            <input class="inp" bind:value={config.theme.active_border_color} placeholder="rgba(255,255,255,0.1)" />
          </div>
        </div>
        <div class="field">
          <label>Item radius</label>
          <div class="field-body">
            <input type="range" min="0" max="20" step="1"
              bind:value={config.theme.item_border_radius} class="slider" />
            <span class="mono-val">{config.theme.item_border_radius}</span>
            <span class="unit">px</span>
          </div>
        </div>
      </div>

    {:else if tab === "triggers"}

      <p class="hint">Each trigger is a regex. The capture group <code>(.*)</code> becomes the search query.</p>
      <div class="trigger-grid">
        {#each TRIGGER_LABELS as [key, label]}
          <div class="trig-item">
            <span class="trig-label">{label}</span>
            <input class="inp inp-mono" bind:value={config.triggers[key]} placeholder="regex…" />
          </div>
        {/each}
      </div>

    {:else if tab === "web"}

      <p class="hint">Use <code>{"{}"}</code> as the query placeholder in the URL.</p>

      {#each config.web_searches as ws, i}
        <div class="ws-card">
          <div class="ws-card-head">
            <input class="ws-name" bind:value={ws.name} placeholder="Name" />
            <button class="rm-btn" on:click={() => removeWebSearch(i)}>
              Remove
            </button>
          </div>
          <div class="field">
            <label>Trigger</label>
            <div class="field-body">
              <input class="inp inp-mono" bind:value={ws.trigger} placeholder="^keyword\s+(.*)$" />
            </div>
          </div>
          <div class="field" style="margin-bottom:0">
            <label>URL</label>
            <div class="field-body">
              <input class="inp" bind:value={ws.url} placeholder={"https://example.com/search?q={}"} />
            </div>
          </div>
        </div>
      {/each}

      <button class="add-btn" on:click={addWebSearch}>+ Add web search</button>

      <div class="sep" />
      <div class="subsec">Default search</div>
      <p class="hint">Shown when no trigger matches.</p>

      <div class="check-list">
        {#each config.web_searches.filter(w => w.name) as ws}
          <div class="check-row">
            <input
              type="checkbox"
              id="ds-{ws.name}"
              checked={config.default_search.web_searches.includes(ws.name)}
              on:change={() => toggleDefaultSearch(ws.name)}
            />
            <label for="ds-{ws.name}" class="check-label">{ws.name}</label>
          </div>
        {/each}
      </div>

      <div class="field" style="margin-top:12px">
        <label>Max results per source</label>
        <div class="field-body">
          <input type="number" min="1" max="10"
            bind:value={config.default_search.max_web_results} class="inp inp-sm" />
        </div>
      </div>

    {:else if tab === "general"}

      <div class="group">
        <div class="group-label">AI</div>
        <div class="field">
          <label>Groq API key</label>
          <div class="field-body">
            {#if showApiKey}
              <input class="inp" bind:value={groqApiKey} placeholder="gsk_…" />
            {:else}
              <input type="password" class="inp" bind:value={groqApiKey} placeholder="gsk_…" />
            {/if}
            <button class="toggle-btn" on:click={() => (showApiKey = !showApiKey)}>
              {showApiKey ? "hide" : "show"}
            </button>
          </div>
        </div>
        <p class="hint indent">Used for the <code>ai</code> trigger. Get a free key at <code>console.groq.com</code>.</p>
      </div>

      <div class="sep" />

      <div class="group">
        <div class="group-label">Storage</div>
        <div class="field">
          <label>Config path</label>
          <div class="field-body">
            <input class="inp readonly" value="~/.config/quarry/config.toml" readonly />
          </div>
        </div>
      </div>

    {/if}
  </div>

  <div class="footer">
    <button class="save-btn" class:saving on:click={save} disabled={saving}>
      {saving ? "Saving…" : "Save"}
    </button>
  </div>

  <div class="toast-container">
    {#each $toasts as toast (toast.id)}
      <div
        class="toast {toast.type}"
        in:fly={{ y: 16, duration: 350, opacity: 0, easing: backOut }}
        out:fly={{ y: 8, duration: 140, opacity: 0 }}
      >
        <span class="toast-dot {toast.type}" />
        {toast.message}
      </div>
    {/each}
  </div>
</main>

<style>
  :global(html, body) {
    margin: 0; padding: 0; height: 100%; overflow: hidden; background: transparent;
  }

  main {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: #111113;
    color: rgba(255, 255, 255, 0.82);
    font-family: var(--q-sans);
    font-size: var(--q-font-size, 15px);
    overflow: hidden;
  }

  /* ── Tabs ─────────────────────────────────────── */
  .tabs {
    display: flex;
    gap: 1px;
    padding: 10px 14px 0;
    flex-shrink: 0;
    border-bottom: 0.5px solid rgba(255, 255, 255, 0.07);
  }

  .tab {
    background: none;
    border: none;
    color: rgba(255, 255, 255, 0.28);
    padding: 7px 14px 9px;
    border-radius: 6px 6px 0 0;
    font-family: inherit;
    font-size: 1em;
    cursor: pointer;
    transition: color 0.1s;
    white-space: nowrap;
  }

  .tab:hover { color: rgba(255, 255, 255, 0.55); }

  .tab.active {
    color: rgba(255, 255, 255, 0.85);
    background: rgba(255, 255, 255, 0.06);
    border: 0.5px solid rgba(255, 255, 255, 0.1);
    border-bottom: 0.5px solid #111113;
    margin-bottom: -0.5px;
  }

  /* ── Content ──────────────────────────────────── */
  .content {
    flex: 1;
    overflow-y: auto;
    padding: 20px 22px 28px;
  }

  .loading {
    padding: 44px;
    text-align: center;
    color: rgba(255, 255, 255, 0.25);
  }

  /* ── Groups ───────────────────────────────────── */
  .group { margin-bottom: 24px; }

  .group-label {
    font-size: 0.73em;
    font-weight: 500;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    color: rgba(255, 255, 255, 0.25);
    margin-bottom: 12px;
  }

  /* ── Fields ───────────────────────────────────── */
  .field {
    display: flex;
    align-items: center;
    gap: 14px;
    margin-bottom: 10px;
  }

  .field label {
    width: 168px;
    flex-shrink: 0;
    color: rgba(255, 255, 255, 0.42);
    font-size: 1em;
    line-height: 1.4;
  }

  .field-body {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  /* ── Inputs ───────────────────────────────────── */
  .inp {
    flex: 1;
    background: rgba(255, 255, 255, 0.05);
    border: 0.5px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    padding: 6px 9px;
    color: rgba(255, 255, 255, 0.82);
    font-family: var(--q-mono);
    font-size: 1em;
    outline: none;
    transition: border-color 0.12s;
    min-width: 0;
  }

  .inp:focus { border-color: rgba(255, 255, 255, 0.28); }
  .inp.readonly { opacity: 0.4; cursor: default; font-family: var(--q-sans); }
  .inp-sm { flex: 0 0 72px; }
  .inp-mono { font-family: var(--q-mono); }

  /* ── Color swatch ─────────────────────────────── */
  .swatch {
    width: 28px;
    height: 28px;
    border-radius: 5px;
    border: 0.5px solid rgba(255, 255, 255, 0.12);
    flex-shrink: 0;
    cursor: pointer;
    position: relative;
    overflow: hidden;
  }

  .swatch input[type="color"] {
    position: absolute;
    inset: -4px;
    width: calc(100% + 8px);
    height: calc(100% + 8px);
    opacity: 0;
    cursor: pointer;
  }

  /* ── Slider ───────────────────────────────────── */
  .slider { flex: 1; accent-color: rgba(255, 255, 255, 0.55); }

  .mono-val {
    font-family: var(--q-mono);
    font-size: 0.8em;
    color: rgba(255, 255, 255, 0.32);
    width: 28px;
    text-align: right;
    flex-shrink: 0;
  }

  .unit {
    font-size: 0.8em;
    color: rgba(255, 255, 255, 0.22);
    flex-shrink: 0;
  }

  /* ── Separator ────────────────────────────────── */
  .sep { height: 0.5px; background: rgba(255, 255, 255, 0.07); margin: 18px 0; }

  /* ── Hint ─────────────────────────────────────── */
  .hint {
    color: rgba(255, 255, 255, 0.25);
    font-size: 0.8em;
    line-height: 1.6;
    margin-bottom: 14px;
  }

  .hint.indent { padding-left: 182px; margin-top: 6px; }

  .hint code {
    font-family: var(--q-mono);
    background: rgba(255, 255, 255, 0.07);
    padding: 1px 5px;
    border-radius: 4px;
    font-size: 0.8em;
  }

  /* ── Triggers ─────────────────────────────────── */
  .trigger-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px 20px; }

  .trig-item { display: flex; flex-direction: column; gap: 4px; }

  .trig-label { font-size: 0.8em; color: rgba(255, 255, 255, 0.35); }

  /* ── Web searches ─────────────────────────────── */
  .ws-card {
    background: rgba(255, 255, 255, 0.03);
    border: 0.5px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    padding: 14px;
    margin-bottom: 10px;
  }

  .ws-card-head { display: flex; align-items: center; gap: 10px; margin-bottom: 12px; }

  .ws-name {
    flex: 1;
    background: none;
    border: none;
    border-bottom: 0.5px solid rgba(255, 255, 255, 0.1);
    border-radius: 0;
    padding: 4px 0;
    color: rgba(255, 255, 255, 0.82);
    font-family: var(--q-sans);
    font-size: 1em;
    font-weight: 500;
    outline: none;
    transition: border-color 0.12s;
  }

  .ws-name:focus { border-bottom-color: rgba(255, 255, 255, 0.3); }
  .ws-name::placeholder { color: rgba(255, 255, 255, 0.2); }

  .rm-btn {
    background: none;
    border: 0.5px solid rgba(255, 80, 80, 0.2);
    color: rgba(255, 100, 100, 0.45);
    font-family: var(--q-sans);
    font-size: 0.8em;
    padding: 4px 9px;
    border-radius: 5px;
    cursor: pointer;
    transition: background 0.1s, color 0.1s;
    white-space: nowrap;
  }

  .rm-btn:hover { background: rgba(255, 80, 80, 0.07); color: rgba(255, 120, 120, 0.8); }

  .add-btn {
    background: rgba(255, 255, 255, 0.03);
    border: 0.5px dashed rgba(255, 255, 255, 0.12);
    color: rgba(255, 255, 255, 0.35);
    font-family: var(--q-sans);
    font-size: 1em;
    padding: 9px;
    border-radius: 7px;
    width: 100%;
    cursor: pointer;
    transition: background 0.1s, color 0.1s;
    margin-top: 4px;
  }

  .add-btn:hover { background: rgba(255, 255, 255, 0.06); color: rgba(255, 255, 255, 0.6); }

  .subsec {
    font-size: 0.73em;
    font-weight: 500;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    color: rgba(255, 255, 255, 0.25);
    margin: 18px 0 10px;
  }

  .check-list { display: flex; flex-direction: column; gap: 8px; }

  .check-row { display: flex; align-items: center; gap: 9px; }

  .check-row input[type="checkbox"] {
    width: 14px;
    height: 14px;
    accent-color: rgba(255, 255, 255, 0.6);
    flex-shrink: 0;
  }

  .check-label { font-size: 1em; color: rgba(255, 255, 255, 0.55); }

  /* ── General ──────────────────────────────────── */
  .toggle-btn {
    background: rgba(255, 255, 255, 0.05);
    border: 0.5px solid rgba(255, 255, 255, 0.1);
    color: rgba(255, 255, 255, 0.4);
    font-family: var(--q-mono);
    font-size: 0.8em;
    padding: 0 10px;
    height: 30px;
    border-radius: 6px;
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.1s;
    flex-shrink: 0;
  }

  .toggle-btn:hover { background: rgba(255, 255, 255, 0.09); }

  /* ── Footer ───────────────────────────────────── */
  .footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    padding: 10px 16px;
    border-top: 0.5px solid rgba(255, 255, 255, 0.07);
    flex-shrink: 0;
  }

  .save-btn {
    background: rgba(255, 255, 255, 0.08);
    border: 0.5px solid rgba(255, 255, 255, 0.15);
    color: rgba(255, 255, 255, 0.8);
    font-family: var(--q-sans);
    font-size: 1em;
    font-weight: 500;
    padding: 6px 20px;
    border-radius: 7px;
    cursor: pointer;
    transition: background 0.12s;
  }

  .save-btn:hover:not(:disabled) { background: rgba(255, 255, 255, 0.13); }
  .save-btn:disabled, .save-btn.saving { opacity: 0.45; cursor: default; }

  /* ── Toasts ───────────────────────────────────── */
  .toast-container {
    position: fixed;
    bottom: 14px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    flex-direction: column;
    gap: 6px;
    align-items: center;
    pointer-events: none;
    z-index: 1000;
  }

  .toast {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 18px;
    border-radius: 999px;
    font-size: 1em;
    white-space: nowrap;
    color: rgba(255, 255, 255, 0.7);
    background: rgba(5, 5, 5, 0.6);
    border: 0.5px solid rgba(255, 255, 255, 0.12);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
  }

  .toast-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .toast-dot.success { background: #4ade80; }
  .toast-dot.error   { background: #f87171; }
  .toast-dot.info    { background: #60a5fa; }
  .toast.error  { border-color: rgba(248, 113, 113, 0.2); }
  .toast.info   { border-color: rgba(96, 165, 250, 0.2); }
</style>
