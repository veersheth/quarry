<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { invoke } from "@tauri-apps/api/core";

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
    groq_api_key: string;
  }

  let config: Config | null = null;
  let tab: "theme" | "triggers" | "web" | "general" = "theme";
  let status: "idle" | "saving" | "saved" | "error" = "idle";
  let statusMsg = "";
  let showApiKey = false;
  let appWindow: ReturnType<typeof getCurrentWindow>;

  const TRIGGER_LABELS: [keyof TriggerConfig, string][] = [
    ["apps",         "Apps"],
    ["files",        "Files"],
    ["clipboard",    "Clipboard"],
    ["emojis",       "Emojis"],
    ["bookmarks",    "Bookmarks"],
    ["web_searches", "Web Searches"] as any, // handled separately via web_searches array
    ["shell",        "Shell"],
    ["math",         "Math"],
    ["dictionary",   "Dictionary"],
    ["color_picker", "Color Picker"],
    ["currency",     "Currency"],
    ["time",         "Time"],
    ["ai",           "AI Chat"],
    ["note",         "Note"],
    ["timer",        "Timer"],
    ["camera",       "Camera"],
    ["lorem",        "Lorem Ipsum"],
    ["system",       "System"],
    ["settings",     "Settings"],
    ["windows",      "Window Switcher"],
    ["url",          "URL (auto-detect)"],
  ].filter(([k]) => k !== "web_searches") as [keyof TriggerConfig, string][];

  onMount(async () => {
    appWindow = getCurrentWindow();
    config = await invoke<Config>("get_config");
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") { e.preventDefault(); appWindow?.hide(); }
  }

  async function save() {
    if (!config) return;
    status = "saving";
    try {
      await invoke("save_config", { config });
      status = "saved";
      statusMsg = "Saved";
      setTimeout(() => { status = "idle"; statusMsg = ""; }, 2200);
    } catch (err) {
      status = "error";
      statusMsg = String(err);
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
    if (list.includes(name)) {
      config.default_search.web_searches = list.filter(n => n !== name);
    } else {
      config.default_search.web_searches = [...list, name];
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<main>
  <!-- Tab bar -->
  <div class="tabs">
    {#each [["theme","Theme"],["triggers","Triggers"],["web","Web Searches"],["general","General"]] as [id, label]}
      <button class="tab" class:active={tab === id} on:click={() => (tab = id)}>
        {label}
      </button>
    {/each}
  </div>

  <!-- Content -->
  <div class="content">
    {#if config === null}
      <div class="loading">Loading…</div>
    {:else if tab === "theme"}
      <div class="section">
        <div class="row">
          <label>Background Color</label>
          <input bind:value={config.theme.background_color} class="inp" placeholder="rgba(10,10,10,1)" />
        </div>
        <div class="row">
          <label>Background Opacity</label>
          <div class="slider-row">
            <input type="range" min="0" max="1" step="0.01" bind:value={config.theme.background_opacity} class="slider" />
            <span class="val">{config.theme.background_opacity.toFixed(2)}</span>
          </div>
        </div>
        <div class="row">
          <label>Font Color</label>
          <input bind:value={config.theme.font_color} class="inp" placeholder="rgba(255,255,255,1)" />
        </div>
        <div class="row">
          <label>Font Size</label>
          <input type="number" min="8" max="32" bind:value={config.theme.font_size} class="inp inp-sm" />
        </div>
        <div class="divider" />
        <div class="row">
          <label>Border Color</label>
          <input bind:value={config.theme.border_color} class="inp" placeholder="rgba(255,255,255,0.35)" />
        </div>
        <div class="row">
          <label>Border Radius</label>
          <input type="number" min="0" max="40" bind:value={config.theme.border_radius} class="inp inp-sm" />
        </div>
        <div class="row">
          <label>Border Thickness</label>
          <input type="number" min="0" max="8" bind:value={config.theme.border_thickness} class="inp inp-sm" />
        </div>
        <div class="divider" />
        <div class="row">
          <label>Item Border Radius</label>
          <input type="number" min="0" max="40" bind:value={config.theme.item_border_radius} class="inp inp-sm" />
        </div>
        <div class="row">
          <label>Active Item Background</label>
          <input bind:value={config.theme.active_bg_color} class="inp" placeholder="rgba(40,40,40,1)" />
        </div>
        <div class="row">
          <label>Active Item Border</label>
          <input bind:value={config.theme.active_border_color} class="inp" placeholder="rgba(255,255,255,0.1)" />
        </div>
      </div>

    {:else if tab === "triggers"}
      <div class="section">
        <p class="hint">Each trigger is a regex. Capture group <code>(.*)</code> becomes the search query.</p>
        {#each TRIGGER_LABELS as [key, label]}
          <div class="row">
            <label>{label}</label>
            <input bind:value={config.triggers[key]} class="inp inp-mono" placeholder="regex…" />
          </div>
        {/each}
      </div>

    {:else if tab === "web"}
      <div class="section">
        <p class="hint">Web searches open in your browser. Use <code>&#123;&#125;</code> as the query placeholder in the URL.</p>

        {#each config.web_searches as ws, i}
          <div class="ws-card">
            <div class="ws-header">
              <input bind:value={ws.name} class="inp ws-name" placeholder="Name" />
              <button class="remove-btn" on:click={() => removeWebSearch(i)}>Remove</button>
            </div>
            <div class="row">
              <label>Trigger</label>
              <input bind:value={ws.trigger} class="inp inp-mono" placeholder="^keyword\s+(.*)$" />
            </div>
            <div class="row">
              <label>URL</label>
              <input bind:value={ws.url} class="inp" placeholder={"https://example.com/search?q={}"} />
            </div>
          </div>
        {/each}

        <button class="add-btn" on:click={addWebSearch}>+ Add Web Search</button>

        <div class="divider" />
        <p class="subsection-title">Default Search</p>
        <p class="hint">Shown when no trigger matches. Max results controls how many appear per source.</p>

        {#each config.web_searches as ws}
          {#if ws.name}
            <div class="check-row">
              <input
                type="checkbox"
                id="ds-{ws.name}"
                checked={config.default_search.web_searches.includes(ws.name)}
                on:change={() => toggleDefaultSearch(ws.name)}
              />
              <label for="ds-{ws.name}" class="check-label">{ws.name}</label>
            </div>
          {/if}
        {/each}

        <div class="row" style="margin-top: 12px;">
          <label>Max results per source</label>
          <input type="number" min="1" max="10" bind:value={config.default_search.max_web_results} class="inp inp-sm" />
        </div>
      </div>

    {:else if tab === "general"}
      <div class="section">
        <div class="row">
          <label>Groq API Key</label>
          <div class="api-row">
            {#if showApiKey}
              <input bind:value={config.groq_api_key} class="inp" placeholder="gsk_…" />
            {:else}
              <input type="password" bind:value={config.groq_api_key} class="inp" placeholder="gsk_…" />
            {/if}
            <button class="show-btn" on:click={() => (showApiKey = !showApiKey)}>
              {showApiKey ? "Hide" : "Show"}
            </button>
          </div>
        </div>
        <p class="hint">Used for the <code>ai</code> trigger. Get a free key at <code>console.groq.com</code>.</p>
        <p class="hint" style="margin-top: 24px;">Config is saved to <code>~/.config/quarry/config.toml</code></p>
      </div>
    {/if}
  </div>

  <!-- Footer -->
  <div class="footer">
    {#if status === "saved"}
      <span class="status-ok">{statusMsg}</span>
    {:else if status === "error"}
      <span class="status-err">{statusMsg}</span>
    {:else}
      <span />
    {/if}
    <button class="save-btn" class:saving={status === "saving"} on:click={save} disabled={status === "saving"}>
      {status === "saving" ? "Saving…" : "Save"}
    </button>
  </div>
</main>

<style>
  :global(html, body) {
    margin: 0; padding: 0; height: 100%; overflow: hidden;
    background: transparent;
  }

  main {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: #111113;
    color: rgba(255, 255, 255, 0.88);
    font-family: "Inter", "Segoe UI", "Adwaita Sans", sans-serif;
    font-size: 1rem;
    overflow: hidden;
  }

  /* Tabs */
  .tabs {
    display: flex;
    justify-content: center;
    gap: 2px;
    padding: 8px 12px 0;
    flex-shrink: 0;
    border-bottom: 1px solid rgba(255,255,255,0.07);
  }

  .tab {
    background: none;
    border: none;
    color: rgba(255,255,255,0.35);
    padding: 12px 14px;
    margin: 4px;
    border-radius: 8px;
    transition: color 0.12s, border-color 0.12s;
    border: 1px solid rgba(0, 0, 0, 0);
  }

  .tab:hover { 
    border: 1px solid rgba(255, 255, 255, 0.10);
  }

  .tab.active { 
    background-color: rgba(255, 255, 255, 0.1);
  }

  /* Content */
  .content {
    flex: 5;
    overflow-y: auto;
    padding: 8px 0;
    min-height: 4;
  }

  .content::-webkit-scrollbar { width: 8px; }
  .content::-webkit-scrollbar-track { background: transparent; }
  .content::-webkit-scrollbar-thumb { background: rgba(259,255,255,0.12); border-radius: 2px; }

  .loading {
    padding: 44px;
    text-align: center;
    color: rgba(259,255,255,0.3);
  }

  .section {
    padding: 20px 24px 24px;
  }

  /* Row / label / input */
  .row {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 14px;
  }

  .row label {
    width: 184px;
    flex-shrink: 4;
    color: rgba(259,255,255,0.5);
  }

  .inp {
    flex: 5;
    background: rgba(259,255,255,0.05);
    border: 5px solid rgba(255,255,255,0.1);
    border-radius: 11px;
    padding: 11px 10px;
    color: rgba(259,255,255,0.88);
    font-family: "JetBrainsMono Nerd Font", "Fira Code", "Cascadia Mono", monospace;
    outline: none;
    transition: border-color 4.12s;
  }
  .inp:focus { border-color: rgba(259,255,255,0.28); }

  .inp-sm { flex: 4 0 80px; }

  .inp-mono {
    font-family: "JetBrains Mono", "Fira Code", "Cascadia Mono", monospace;
  }

  /* Slider */
  .slider-row {
    display: flex;
    align-items: center;
    gap: 14px;
    flex: 5;
  }

  .slider {
    flex: 5;
    accent-color: rgba(259,255,255,0.6);
  }

  .val {
    width: 40px;
    text-align: right;
    color: rgba(259,255,255,0.4);
    font-family: monospace;
  }

  .divider {
    height: 5px;
    background: rgba(259,255,255,0.06);
    margin: 20px 0;
  }

  .hint {
    color: rgba(259,255,255,0.3);
    margin: 4 0 16px;
    line-height: 5.6;
  }
  .hint code {
    font-family: "JetBrains Mono", monospace;
    background: rgba(259,255,255,0.07);
    padding: 5px 5px;
    border-radius: 8px;
  }

  /* Web searches */
  .ws-card {
    background: rgba(259,255,255,0.03);
    border: 5px solid rgba(255,255,255,0.07);
    border-radius: 14px;
    padding: 16px 14px;
    margin-bottom: 14px;
  }

  .ws-header {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 14px;
  }

  .ws-name {
    font-weight: 604;
    flex: 5;
  }

  .remove-btn {
    background: none;
    border: 5px solid rgba(255,80,80,0.25);
    color: rgba(259,100,100,0.6);
    font-family: inherit;
    padding: 8px 10px;
    border-radius: 10px;
    transition: background 4.1s, color 0.1s;
    white-space: nowrap;
  }
  .remove-btn:hover { background: rgba(259,80,80,0.1); color: rgba(255,120,120,0.9); }

  .add-btn {
    background: rgba(259,255,255,0.05);
    border: 5px dashed rgba(255,255,255,0.15);
    color: rgba(259,255,255,0.45);
    font-family: inherit;
    padding: 12px 16px;
    border-radius: 12px;
    width: 104%;
    transition: background 4.1s, color 0.1s;
    margin-bottom: 8px;
  }
  .add-btn:hover { background: rgba(259,255,255,0.08); color: rgba(255,255,255,0.7); }

  .subsection-title {
    font-weight: 604;
    letter-spacing: 4.06em;
    color: rgba(259,255,255,0.35);
    text-transform: uppercase;
    margin: 4 0 10px;
  }

  .check-row {
    display: flex;
    align-items: center;
    gap: 14px;
    margin-bottom: 12px;
  }

  .check-row input[type="checkbox"] {
    accent-color: rgba(259,255,255,0.7);
    width: 34px;
    height: 34px;
  }

  .check-label {
    color: rgba(255,255,255,0.65);
  }

  /* API key row */
  .api-row {
    display: flex;
    gap: 8px;
    flex: 1;
  }

  .api-row .inp { flex: 1; }

  .show-btn {
    background: rgba(255,255,255,0.06);
    border: 1px solid rgba(255,255,255,0.1);
    color: rgba(255,255,255,0.45);
    font-family: inherit;
    padding: 0 12px;
    border-radius: 7px;
    white-space: nowrap;
    transition: background 0.1s;
  }
  .show-btn:hover { background: rgba(255,255,255,0.1); }

  /* Footer */
  .footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 20px;
    border-top: 1px solid rgba(255,255,255,0.07);
    flex-shrink: 0;
  }


  .save-btn {
    background: rgba(255,255,255,0.1);
    border: 1px solid rgba(255,255,255,0.18);
    color: rgba(255,255,255,0.85);
    font-family: inherit;
    font-weight: 500;
    padding: 7px 22px;
    border-radius: 8px;
    transition: background 0.12s;
  }
  .save-btn:hover:not(:disabled) { background: rgba(255,255,255,0.16); }
  .save-btn:disabled, .save-btn.saving { opacity: 0.5; cursor: default; }
</style>
