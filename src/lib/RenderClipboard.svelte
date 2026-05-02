<script lang="ts">
  import { writable, type Writable } from "svelte/store";
  import type { ResultItem } from "../stores/search";
  import { runItemAction } from "./keyHandler";

  export let listitems: {
    name: string;
    actions: { id: string; name: string }[];
    description?: string;
    icon?: string;
    thumbnail?: string;
    ocr_text?: string;
    pinned?: boolean;
  }[] = [];
  export let activeIndex: Writable<number> = writable(0);
  export let onContextMenu:
    | ((e: MouseEvent, item: (typeof listitems)[number]) => void)
    | undefined = undefined;

  $: activeItem = listitems[$activeIndex];
  $: activeColor = activeItem?.thumbnail ? null : getValidColor(activeItem?.name);
  $: contentType = activeItem ? detectType(activeItem) : ("text" as ContentType);
  $: footerMeta = activeItem ? getFooterMeta(activeItem, contentType) : [];

  let showOcrText = false;
  $: if (activeItem) showOcrText = false;

  type UrlMeta = { favicon: string | null; thumbnail: string | null; title: string | null };

  const urlMetaCache = new Map<string, UrlMeta | "loading" | "error">();
  let urlMeta: Record<string, UrlMeta | "loading" | "error"> = {};

  function faviconFor(hostname: string): string {
    return `https://www.google.com/s2/favicons?domain=${hostname}&sz=32`;
  }

  function youtubeVideoId(url: string): string | null {
    try {
      const u = new URL(url);
      if (u.hostname === "youtu.be") return u.pathname.slice(1).split("?")[0] || null;
      if (u.hostname.includes("youtube.com")) {
        const v = u.searchParams.get("v");
        if (v) return v;
        const parts = u.pathname.split("/").filter(Boolean);
        const idx = parts.findIndex((p) => ["shorts", "embed", "v"].includes(p));
        if (idx !== -1 && parts[idx + 1]) return parts[idx + 1];
      }
    } catch { /* ignore */ }
    return null;
  }

  function youtubeThumbnail(videoId: string): string {
    return `https://i.ytimg.com/vi/${videoId}/hqdefault.jpg`;
  }

  async function fetchUrlMeta(url: string): Promise<void> {
    if (urlMetaCache.has(url)) return;
    urlMetaCache.set(url, "loading");
    urlMeta = { ...urlMeta, [url]: "loading" };
    try {
      const u = new URL(url);
      const hostname = u.hostname;
      const ytId = youtubeVideoId(url);
      if (ytId) {
        const meta: UrlMeta = { favicon: faviconFor(hostname), thumbnail: youtubeThumbnail(ytId), title: null };
        urlMetaCache.set(url, meta);
        urlMeta = { ...urlMeta, [url]: meta };
        return;
      }
      const meta: UrlMeta = { favicon: faviconFor(hostname), thumbnail: null, title: null };
      urlMetaCache.set(url, meta);
      urlMeta = { ...urlMeta, [url]: meta };
    } catch {
      urlMetaCache.set(url, "error");
      urlMeta = { ...urlMeta, [url]: "error" };
    }
  }

  $: {
    for (const item of listitems) {
      if (isURL(item.name) && !urlMetaCache.has(item.name)) fetchUrlMeta(item.name);
    }
  }

  function getUrlMeta(url: string): UrlMeta | null {
    const m = urlMeta[url];
    if (!m || m === "loading" || m === "error") return null;
    return m;
  }

  function handleClick(item: typeof listitems[0]) {
    runItemAction(item as unknown as ResultItem);
  }

  function truncate(str: string | undefined, maxLength: number): string {
    if (!str) return "";
    return str.length > maxLength ? str.slice(0, maxLength) + "…" : str;
  }

  function formatTimestamp(timestamp?: string | number): string {
    if (!timestamp) return "";
    const ts = typeof timestamp === "string" ? Number(timestamp) : timestamp;
    if (isNaN(ts)) return String(timestamp);
    const now = Date.now() / 1000;
    const age = now - ts;
    if (age < 60) return "just now";
    if (age < 3600) return `${Math.floor(age / 60)}m ago`;
    if (age < 86400) return `${Math.floor(age / 3600)}h ago`;
    return `${Math.floor(age / 86400)}d ago`;
  }

  function getValidColor(str: string | undefined): string | null {
    if (!str) return null;
    const trimmed = str.trim();
    if (/^(#([A-Fa-f0-9]{3,4}){1,2}|(rgb|hsl)a?\s*\(.*\))$/i.test(trimmed)) return trimmed;
    if (/^(\d{1,3}),\s*(\d{1,3}),\s*(\d{1,3})(,\s*[\d.]+)?$/.test(trimmed)) return `rgb(${trimmed})`;
    if (/^(\d{1,3})°?,\s*(\d{1,3})%,\s*(\d{1,3})%(,\s*[\d.]+)?$/.test(trimmed)) return `hsl(${trimmed.replace("°", "")})`;
    return null;
  }

  type ContentType = "image" | "color" | "url" | "email" | "json" | "code" | "multiline" | "text";

  function detectType(item: typeof listitems[0]): ContentType {
    if (item.thumbnail) return "image";
    const v = item.name?.trim() ?? "";
    if (getValidColor(v)) return "color";
    if (isURL(v)) return "url";
    if (isEmail(v)) return "email";
    if (isJSON(v)) return "json";
    if (isCode(v)) return "code";
    if (v.includes("\n")) return "multiline";
    return "text";
  }

  function isURL(v: string): boolean {
    try {
      const u = new URL(v);
      return u.protocol === "http:" || u.protocol === "https:" || u.protocol === "ftp:";
    } catch { return false; }
  }

  function isEmail(v: string): boolean {
    return /^[^\s@]+@[^\s@]+\.[^\s@]{2,}$/.test(v);
  }

  function isJSON(v: string): boolean {
    if (!/^[\[{]/.test(v.trim())) return false;
    try { JSON.parse(v); return true; } catch { return false; }
  }

  const CODE_PATTERNS = [
    /^(import|export|from|require|const|let|var|function|class|def|fn|pub|use|#include|package|namespace)\s/m,
    /^\s*(if|for|while|switch|match|try|catch)\s*[\(\{]/m,
    /[;{}]\s*\n/,
    /\/\/.*\n|\/\*[\s\S]*?\*\//,
    /^\s{2,}|\t/m,
  ];

  function isCode(v: string): boolean {
    if (!v.includes("\n")) return false;
    return CODE_PATTERNS.filter((p) => p.test(v)).length >= 2;
  }

  function parseURL(v: string) {
    try {
      const u = new URL(v);
      return { hostname: u.hostname, path: u.pathname + u.search + u.hash, display: u.hostname.replace(/^www\./, "") };
    } catch { return null; }
  }

  function prettyJSON(v: string): string {
    try { return JSON.stringify(JSON.parse(v), null, 2); } catch { return v; }
  }

  function jsonStats(v: string): { keys: number; depth: number; type: string } {
    try {
      const parsed = JSON.parse(v);
      const type = Array.isArray(parsed) ? "array" : typeof parsed;
      const keys = type === "array" ? parsed.length : Object.keys(parsed).length;
      function maxDepth(o: unknown, d = 0): number {
        if (typeof o !== "object" || o === null) return d;
        const values = Object.values(o as Record<string, unknown>);
        if (values.length === 0) return d;
        return Math.max(...values.map((v) => maxDepth(v, d + 1)));
      }
      return { keys, depth: maxDepth(parsed), type };
    } catch { return { keys: 0, depth: 0, type: "unknown" }; }
  }

  function guessLang(v: string): string {
    if (/^\s*(import|export|from|const|let|var|=>|function|class)\s/m.test(v)) return "js/ts";
    if (/^\s*(def |import |from .* import|class .*:)/m.test(v)) return "python";
    if (/^\s*(fn |pub |use |let mut|impl |struct )/m.test(v)) return "rust";
    if (/^\s*(func |package |import ")/m.test(v)) return "go";
    if (/#include|std::|cout|cin/.test(v)) return "c/c++";
    if (/<\?php|echo\s/.test(v)) return "php";
    if (/^\s*(SELECT|INSERT|UPDATE|DELETE|FROM|WHERE)\s/im.test(v)) return "sql";
    if (/^\s*(<[a-z][\w-]*[\s>]|<\/[a-z])/im.test(v)) return "html";
    if (/^\s*[\w-]+\s*:\s*[\w#"'].*;?$/m.test(v)) return "css";
    if (/^\s*[\w]+:\s*\n|^---/m.test(v)) return "yaml";
    return "code";
  }

  function textStats(v: string) {
    const words = v.trim().split(/\s+/).filter(Boolean).length;
    return { lines: v.split("\n").length, words, chars: v.length };
  }

  function getFooterMeta(item: typeof listitems[0], type: ContentType): string[] {
    const v = item.name ?? "";
    switch (type) {
      case "code": {
        const s = textStats(v);
        return [guessLang(v), `${s.lines} lines`];
      }
      case "json": {
        const s = jsonStats(v);
        return [s.type, `${s.keys} ${s.type === "array" ? "items" : "keys"}`, `depth ${s.depth}`];
      }
      case "multiline":
      case "text": {
        const s = textStats(v);
        const parts = [`${s.words} words`, `${s.chars} chars`];
        if (s.lines > 1) parts.unshift(`${s.lines} lines`);
        return parts;
      }
      case "url": {
        const parsed = parseURL(v);
        const ytId = youtubeVideoId(v);
        const parts = parsed ? [parsed.display] : [];
        if (ytId) parts.push("youtube");
        return parts;
      }
      case "email": {
        const [local, domain] = v.split("@");
        return [local, domain];
      }
      case "color":
        return [v];
      default:
        return [];
    }
  }
</script>

<div class="clipboard">
  <!-- Left: scrollable item list -->
  <div class="result-list">
    {#each listitems as item, index}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="result-item"
        class:active={index === $activeIndex}
        class:pinned={item.pinned}
        on:mouseenter={() => activeIndex.set(index)}
        on:click={() => handleClick(item)}
        on:contextmenu={(e) => { e.preventDefault(); onContextMenu?.(e, item); }}
      >
        <!-- Type icon / thumbnail -->
        <div class="item-icon">
          {#if item.thumbnail}
            <img class="icon-thumb" src={item.thumbnail} alt="" />
          {:else if getValidColor(item.name)}
            <div class="icon-swatch" style:background-color={getValidColor(item.name)}></div>
          {:else if isURL(item.name)}
            {@const favicon = getUrlMeta(item.name)?.favicon}
            {#if favicon}
              <img class="icon-favicon" src={favicon} alt=""
                on:error={(e) => {
                  const img = e.target as HTMLImageElement;
                  img.style.display = "none";
                  (img.nextElementSibling as HTMLElement | null)?.style.setProperty("display", "flex");
                }}
              />
              <div class="icon-pill icon-url" style="display:none">url</div>
            {:else}
              <div class="icon-pill icon-url">url</div>
            {/if}
          {:else if isEmail(item.name)}
            <div class="icon-pill icon-email">@</div>
          {:else if isJSON(item.name)}
            <div class="icon-pill icon-json">{"{}"}</div>
          {:else if isCode(item.name)}
            <div class="icon-pill icon-code">&lt;/&gt;</div>
          {/if}
        </div>

        <!-- Name + description -->
        <div class="item-body">
          <span class="item-name">{truncate(item.name, 26)}</span>
          {#if item.description}
            <span class="item-desc">{truncate(item.description, 30)}</span>
          {/if}
        </div>

        {#if item.pinned}
          <div class="pin-indicator" title="pinned"></div>
        {/if}
      </div>
    {/each}
  </div>

  <!-- Right: preview panel -->
  <div class="info-panel">
    {#if activeItem}
      {#if contentType === "image" && activeItem.ocr_text}
        <div class="view-tabs">
          <button class="view-tab" class:active={!showOcrText} on:click={() => showOcrText = false}>
            Image
          </button>
          <button class="view-tab" class:active={showOcrText} on:click={() => showOcrText = true}>
            Text
          </button>
        </div>
      {/if}

      <div class="preview-area" class:image-fullsize={contentType === "image"}>

        {#if contentType === "image"}
          {#if activeItem.ocr_text && showOcrText}
            <div class="image-ocr-text">{activeItem.ocr_text}</div>
          {:else}
            <img class="image-preview" src={activeItem.thumbnail} alt={activeItem.name} />
          {/if}

        {:else if contentType === "color"}
          <div class="color-hero">
            <div class="checkerboard">
              <div class="main-swatch" style:background-color={activeColor}></div>
            </div>
            <code class="color-value">{activeItem.name}</code>
          </div>

        {:else if contentType === "url"}
          {@const parsed = parseURL(activeItem.name)}
          {@const meta = getUrlMeta(activeItem.name)}
          {@const ytId = youtubeVideoId(activeItem.name)}
          {#if parsed}
            <div class="url-card">
              <div class="url-card-main">
                <div class="url-header">
                  {#if meta?.favicon}
                    <img class="url-favicon" src={meta.favicon} alt=""
                      on:error={(e) => { (e.target as HTMLImageElement).style.display = "none"; }}
                    />
                  {/if}
                  <span class="url-hostname">{parsed.display}</span>
                </div>
                <div class="url-full">{activeItem.name}</div>
                <!-- svelte-ignore a11y_invalid_attribute -->
                <a class="url-open" href={activeItem.name} target="_blank" rel="noopener">open in browser ↗</a>
              </div>
              {#if meta?.thumbnail}
                <div class="url-thumb-wrap">
                  <img class="url-thumb" src={meta.thumbnail} alt="thumbnail"
                    on:error={(e) => {
                      const img = e.target as HTMLImageElement;
                      if (img.parentElement) img.parentElement.style.display = "none";
                    }}
                  />
                </div>
              {/if}
            </div>
          {/if}

        {:else if contentType === "email"}
          <div class="email-card">
            <div class="email-icon">✉</div>
            <div class="email-address">{activeItem.name}</div>
            <a class="url-open" href="mailto:{activeItem.name}">compose email ↗</a>
          </div>

        {:else if contentType === "json"}
          <div class="scroll-container">
            <pre class="code-block json-block">{prettyJSON(activeItem.name)}</pre>
          </div>

        {:else if contentType === "code"}
          <div class="scroll-container">
            <pre class="code-block">{activeItem.name}</pre>
          </div>

        {:else}
          <div class="scroll-container">
            <div class="text-block">{activeItem.name}</div>
          </div>
        {/if}

      </div>

      <!-- Footer: type badge + stats + timestamp -->
      <div class="metadata">
        <span class="type-badge type-{contentType}">{contentType}</span>
        {#each footerMeta as chip}
          <span class="stat-chip">{chip}</span>
        {/each}
        <span class="timestamp">{formatTimestamp(activeItem.description)}</span>
      </div>
    {:else}
      <div class="empty-state">select an item to preview</div>
    {/if}
  </div>
</div>

<style>
  /* ── Layout ──────────────────────────────────────────── */
  .clipboard {
    display: flex;
    height: 100%;
    color: #eee;
  }

  /* ── Left list ───────────────────────────────────────── */
  .result-list {
    flex: 0 0 230px;
    border-right: 1px solid #2a2a2a;
    overflow-y: auto;
    padding: 10px 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .result-item {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 7px 9px;
    border-radius: var(--q-item-border-radius);
    cursor: pointer;
    border: 1.5px solid transparent;
    min-height: 44px;
  }

  .result-item.pinned        { border-color: rgba(168, 85, 247, 0.35); }
  .result-item.active        { background: var(--q-active-bg-color); border-color: var(--q-active-border-color); }
  .result-item.pinned.active { border-color: rgba(192, 132, 252, 0.75); }

  /* Icon cell */
  .item-icon {
    flex-shrink: 0;
    width: 32px;
    height: 32px;
    border-radius: 7px;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .icon-thumb {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .icon-favicon {
    width: 18px;
    height: 18px;
    object-fit: contain;
    border-radius: 3px;
  }

  .icon-swatch {
    width: 100%;
    height: 100%;
    border-radius: 50%;
  }

  .icon-pill {
    font-size: 0.6rem;
    font-family: 'JetBrainsMono Nerd Font', 'Fira Code', monospace;
    padding: 2px 5px;
    border-radius: 5px;
    font-weight: 600;
    text-transform: uppercase;
    white-space: nowrap;
  }
  .icon-url   { background: #1a2a3a; color: #60a5fa; border: 1px solid #2a3a4a; }
  .icon-email { background: #2a1a3a; color: #c084fc; border: 1px solid #3a2a4a; }
  .icon-json  { background: #3a2a1a; color: #fb923c; border: 1px solid #4a3a2a; }
  .icon-code  { background: #2a2a1a; color: #facc15; border: 1px solid #3a3a2a; }

  /* Text cell */
  .item-body {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .item-name {
    display: block;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 0.9rem;
    line-height: 1.3;
  }

  .item-desc {
    display: block;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 0.72rem;
    color: #666;
    line-height: 1.3;
  }

  /* Pin dot */
  .pin-indicator {
    flex-shrink: 0;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: rgba(168, 85, 247, 0.7);
  }

  /* ── Right panel ─────────────────────────────────────── */
  .info-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .preview-area {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
    overflow: hidden;
    min-height: 0;
  }

  .preview-area.image-fullsize {
    padding: 0;
  }

  .image-preview {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  .image-ocr-text {
    width: 100%;
    height: 100%;
    overflow-y: auto;
    padding: 16px 20px;
    font-size: 0.85rem;
    line-height: 1.7;
    color: #ccc;
    white-space: pre-wrap;
    word-break: break-word;
    user-select: text;
  }

  .view-tabs {
    display: flex;
    flex-shrink: 0;
    gap: 2px;
    padding: 6px 10px;
    border-bottom: 1px solid #222;
    color: #555;
  }

  .view-tab {
    flex: 1;
    padding: 5px 0;
    font-size: 0.78rem;
    color: #555;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 6px;
    cursor: pointer;
    letter-spacing: 0.03em;
  }

  .view-tab:hover { color: #999; }

  .view-tab.active {
    color: #ddd;
    background: #1e1e1e;
    border-color: #333;
  }

  /* Shared scroll container for text/code/json */
  .scroll-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
  }

  .text-block {
    flex: 1;
    white-space: pre-wrap;
    word-break: break-word;
    color: #ffb5bc;
    font-size: 0.95rem;
    overflow: auto;
    line-height: 1.6;
  }

  .code-block {
    flex: 1;
    font-family: 'JetBrainsMono Nerd Font', 'Fira Code', monospace;
    font-size: 0.78rem;
    color: #facc15;
    background: #181818;
    border: 1px solid #2e2e2e;
    border-radius: 10px;
    padding: 14px;
    overflow: auto;
    margin: 0;
    white-space: pre;
    line-height: 1.6;
  }

  .json-block { color: #fb923c; }

  .color-hero {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 24px;
  }

  .checkerboard {
    width: 190px;
    height: 190px;
    border-radius: 50%;
    background-image:
      linear-gradient(45deg, #222 25%, transparent 25%),
      linear-gradient(-45deg, #222 25%, transparent 25%),
      linear-gradient(45deg, transparent 75%, #222 75%),
      linear-gradient(-45deg, transparent 75%, #222 75%);
    background-size: 20px 20px;
    background-position: 0 0, 0 10px, 10px -10px, -10px 0;
    background-color: #111;
    overflow: hidden;
    border: 1px solid #333;
  }

  .main-swatch { width: 100%; height: 100%; }

  .color-value {
    font-family: 'JetBrainsMono Nerd Font', 'Fira Code', monospace;
    font-size: 1.1rem;
    background: #222;
    padding: 8px 18px;
    border-radius: 12px;
    color: #eee;
    border: 1px solid #333;
  }

  /* URL card */
  .url-card {
    width: 100%;
    display: flex;
    flex-direction: row;
    gap: 0;
    background: #181818;
    border: 1px solid #2e2e2e;
    border-radius: 14px;
    overflow: hidden;
  }

  .url-card-main {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 20px;
    overflow: hidden;
  }

  .url-thumb-wrap {
    flex: 0 0 40%;
    background: #111;
  }

  .url-thumb {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .url-header {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .url-favicon {
    width: 20px;
    height: 20px;
    border-radius: 4px;
    flex-shrink: 0;
  }

  .url-hostname {
    font-size: 1.05rem;
    font-weight: 600;
    color: #e2e8f0;
  }

  .url-full {
    font-size: 0.78rem;
    color: #60a5fa;
    word-break: break-all;
    font-family: 'JetBrainsMono Nerd Font', 'Fira Code', monospace;
    opacity: 0.8;
  }

  .url-open {
    font-size: 0.78rem;
    color: #fff;
    text-decoration: underline;
    opacity: 0.5;
    align-self: flex-start;
  }
  .url-open:hover { opacity: 1; }

  /* Email card */
  .email-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
    background: #181818;
    border: 1px solid #2e2e2e;
    border-radius: 14px;
    padding: 32px 24px;
    width: 100%;
  }

  .email-icon    { font-size: 2rem; opacity: 0.25; }
  .email-address {
    font-size: 1rem;
    font-family: 'JetBrainsMono Nerd Font', 'Fira Code', monospace;
    color: #c084fc;
    word-break: break-all;
    text-align: center;
  }

  /* Empty state */
  .empty-state {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.85rem;
    color: #444;
  }

  /* ── Footer ──────────────────────────────────────────── */
  .metadata {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 7px 14px;
    border-top: 1px solid #222;
    flex-wrap: wrap;
    flex-shrink: 0;
  }

  .type-badge {
    font-size: 0.7rem;
    padding: 2px 8px;
    border-radius: 6px;
    text-transform: capitalize;
    font-family: 'JetBrainsMono Nerd Font', 'Fira Code', monospace;
  }
  .type-badge.type-text      { background: #252525; color: #777; border: 1px solid #333; }
  .type-badge.type-multiline { background: #252525; color: #777; border: 1px solid #333; }
  .type-badge.type-image     { background: #1a2a1a; color: #6a9; border: 1px solid #2a3a2a; }
  .type-badge.type-color     { background: #2a1a2a; color: #a6a; border: 1px solid #3a2a3a; }
  .type-badge.type-url       { background: #1a2a3a; color: #60a5fa; border: 1px solid #2a3a4a; }
  .type-badge.type-email     { background: #2a1a3a; color: #c084fc; border: 1px solid #3a2a4a; }
  .type-badge.type-json      { background: #3a2a1a; color: #fb923c; border: 1px solid #4a3a2a; }
  .type-badge.type-code      { background: #2a2a1a; color: #facc15; border: 1px solid #3a3a2a; }

  .stat-chip {
    font-size: 0.7rem;
    font-family: 'JetBrainsMono Nerd Font', 'Fira Code', monospace;
    background: #1e1e1e;
    border: 1px solid #2a2a2a;
    color: #555;
    padding: 2px 7px;
    border-radius: 5px;
  }

  .timestamp {
    margin-left: auto;
    font-size: 0.75rem;
    opacity: 0.3;
    font-family: 'JetBrainsMono Nerd Font', 'Fira Code', monospace;
    white-space: nowrap;
  }
</style>
