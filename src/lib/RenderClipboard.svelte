<script lang="ts">
  import { writable, type Writable } from "svelte/store";
  import { invoke } from "@tauri-apps/api/core";
  import { iconSrc } from "./utils";
  import type { ResultItem } from "../stores/search";
  import { mouseHasMoved } from "../stores/search";
  import FooterBar from "./ui/FooterBar.svelte";
  import Chip from "./ui/Chip.svelte";
  import Timestamp from "./ui/Timestamp.svelte";

  const TYPE_ACCENT: Record<string, string> = {
    text:      "#888",
    multiline: "#888",
    image:     "#6aaa99",
    color:     "#aa66aa",
    url:       "#60a5fa",
    email:     "#c084fc",
    json:      "#fb923c",
  };
  import { runItemAction } from "./keyHandler";

  export let listitems: ResultItem[] = [];
  export let activeIndex: Writable<number> = writable(0);
  export let onContextMenu:
    | ((e: MouseEvent, item: ResultItem) => void)
    | undefined = undefined;

  $: activeItem = listitems[$activeIndex];
  $: activeColor = (activeItem && (activeItem.thumbnail || activeItem.thumbnail_key))
    ? null
    : getValidColor(activeItem?.name);
  $: contentType = activeItem
    ? detectType(activeItem)
    : ("text" as ContentType);

  // Cache of thumbnail_key → data URI for list thumbnails
  let thumbCache = new Map<string, string>();
  $: {
    for (const item of listitems) {
      if (item.thumbnail_key && !thumbCache.has(item.thumbnail_key)) {
        const key = item.thumbnail_key;
        invoke<string | null>("get_clipboard_thumbnail", { hash: key })
          .then(t => { if (t) { thumbCache.set(key, t); thumbCache = thumbCache; } })
          .catch(() => {});
      }
    }
  }

  // Lazy-loaded data for the selected item
  let previewImage: string | null = null;
  let previewText: string | null = null;

  $: {
    previewImage = null;
    previewText = null;
    if (activeItem) {
      if (activeItem.thumbnail_key && !activeItem.thumbnail) {
        const imageHash = activeItem.thumbnail_key;
        invoke<string | null>("get_clipboard_image", { hash: imageHash })
          .then(image => {
            if (activeItem?.thumbnail_key === imageHash && image) previewImage = image;
          })
          .catch(() => {});
      } else {
        const copyAction = activeItem.actions.find(a => a.name === "Copy");
        if (copyAction) {
          invoke<string | null>("get_action_text", { actionId: copyAction.id })
            .then(t => { if (t !== null) previewText = t; })
            .catch(() => {});
        }
      }
    }
  }

  $: footerMeta = activeItem ? getFooterMeta(activeItem, contentType, previewText) : [];
  $: imageFooterMeta = activeItem && contentType === "image"
    ? getImageFooterMeta(activeItem, previewImage)
    : [];

  type ClipboardFileMetadata = {
    name: string;
    path: string;
    kind: string;
    extension: string | null;
    size_bytes: number | null;
    modified_unix: number | null;
  };

  let fileMetadata: ClipboardFileMetadata | null = null;

  $: {
    fileMetadata = null;
    if (activeItem && contentType === "file" && previewText) {
      const itemName = activeItem.name;
      const path = previewText.trim();
      invoke<ClipboardFileMetadata | null>("get_clipboard_file_metadata", { path })
        .then((metadata) => {
          if (activeItem?.name === itemName && previewText?.trim() === path) {
            fileMetadata = metadata;
          }
        })
        .catch(() => {});
    }
  }

  function formatFileSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  }

  function formatModifiedTime(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleString();
  }

  function parentPath(path: string): string {
    const separator = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
    return separator > 0 ? path.slice(0, separator) : path;
  }

  function getImageFooterMeta(item: ResultItem, imageData: string | null): string[] {
    const details: string[] = [];
    const dimensions = item.name.match(/(\d+)\s*[×x]\s*(\d+)/i);
    if (dimensions) details.push(`${dimensions[1]} × ${dimensions[2]} px`);
    details.push("PNG");

    if (imageData) {
      const comma = imageData.indexOf(",");
      const header = imageData.slice(0, comma);
      const encoded = imageData.slice(comma + 1);
      if (comma !== -1 && header.includes(";base64")) {
        const padding = encoded.endsWith("==") ? 2 : encoded.endsWith("=") ? 1 : 0;
        const bytes = Math.floor(encoded.length * 3 / 4) - padding;
        details.push(bytes < 1024
          ? `${bytes} B`
          : bytes < 1024 * 1024
            ? `${(bytes / 1024).toFixed(1)} KB`
            : `${(bytes / (1024 * 1024)).toFixed(1)} MB`);
      }
    }

    if (item.ocr_text) details.push(`${item.ocr_text.length.toLocaleString()} chars`);
    return details;
  }

  type UrlMeta = {
    favicon: string | null;
    thumbnail: string | null;
    title: string | null;
  };

  const urlMetaCache = new Map<string, UrlMeta | "loading" | "error">();
  let urlMeta: Record<string, UrlMeta | "loading" | "error"> = {};

  function faviconFor(hostname: string): string {
    return `https://www.google.com/s2/favicons?domain=${hostname}&sz=32`;
  }

  function youtubeVideoId(url: string): string | null {
    try {
      const u = new URL(url);
      if (u.hostname === "youtu.be")
        return u.pathname.slice(1).split("?")[0] || null;
      if (u.hostname.includes("youtube.com")) {
        const v = u.searchParams.get("v");
        if (v) return v;
        const parts = u.pathname.split("/").filter(Boolean);
        const idx = parts.findIndex((p) =>
          ["shorts", "embed", "v"].includes(p),
        );
        if (idx !== -1 && parts[idx + 1]) return parts[idx + 1];
      }
    } catch {
      /* ignore */
    }
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
        const meta: UrlMeta = {
          favicon: faviconFor(hostname),
          thumbnail: youtubeThumbnail(ytId),
          title: null,
        };
        urlMetaCache.set(url, meta);
        urlMeta = { ...urlMeta, [url]: meta };
        return;
      }

      const meta: UrlMeta = {
        favicon: faviconFor(hostname),
        thumbnail: null,
        title: null,
      };
      urlMetaCache.set(url, meta);
      urlMeta = { ...urlMeta, [url]: meta };
    } catch {
      urlMetaCache.set(url, "error");
      urlMeta = { ...urlMeta, [url]: "error" };
    }
  }

  $: {
    for (const item of listitems) {
      if (isURL(item.name) && !urlMetaCache.has(item.name)) {
        fetchUrlMeta(item.name);
      }
    }
  }

  function getUrlMeta(url: string): UrlMeta | null {
    const m = urlMeta[url];
    if (!m || m === "loading" || m === "error") return null;
    return m;
  }

  function getUrlFavicon(url: string): string | null {
    return getUrlMeta(url)?.favicon ?? null;
  }

  function getUrlThumbnail(url: string): string | null {
    return getUrlMeta(url)?.thumbnail ?? null;
  }

  function handleClick(item: ResultItem) {
    runItemAction(item);
  }

  function getValidColor(str: string | undefined): string | null {
    if (!str) return null;
    const trimmed = str.trim();
    const standardRegex = /^(#([A-Fa-f0-9]{3,4}){1,2}|(rgb|hsl)a?\s*\(.*\))$/i;
    if (standardRegex.test(trimmed)) return trimmed;
    const nakedRgb = /^(\d{1,3}),\s*(\d{1,3}),\s*(\d{1,3})(,\s*[\d.]+)?$/;
    if (nakedRgb.test(trimmed)) return `rgb(${trimmed})`;
    const nakedHsl = /^(\d{1,3})°?,\s*(\d{1,3})%,\s*(\d{1,3})%(,\s*[\d.]+)?$/;
    if (nakedHsl.test(trimmed)) return `hsl(${trimmed.replace("°", "")})`;
    return null;
  }

  type ContentType =
    | "image"
    | "file"
    | "color"
    | "url"
    | "email"
    | "json"
    | "multiline"
    | "text";

  function detectType(item: (typeof listitems)[0]): ContentType {
    if (item.thumbnail || item.thumbnail_key) return "image";
    if (isFileItem(item)) return "file";
    const v = item.name?.trim() ?? "";
    if (getValidColor(v)) return "color";
    if (isURL(v)) return "url";
    if (isEmail(v)) return "email";
    if (isJSON(v)) return "json";
    if (v.includes("\n")) return "multiline";
    return "text";
  }

  function isFileItem(item: (typeof listitems)[0]): boolean {
    return item.actions.some((action) =>
      ["Open", "Open Folder", "Open Containing Folder"].includes(action.name),
    );
  }

  function isURL(v: string): boolean {
    try {
      const u = new URL(v);
      return (
        u.protocol === "http:" ||
        u.protocol === "https:" ||
        u.protocol === "ftp:"
      );
    } catch {
      return false;
    }
  }

  function isEmail(v: string): boolean {
    return /^[^\s@]+@[^\s@]+\.[^\s@]{2,}$/.test(v);
  }

  function isJSON(v: string): boolean {
    if (!/^[\[{]/.test(v.trim())) return false;
    try {
      JSON.parse(v);
      return true;
    } catch {
      return false;
    }
  }

  function parseURL(v: string) {
    try {
      const u = new URL(v);
      return {
        protocol: u.protocol.replace(":", "").toUpperCase(),
        hostname: u.hostname,
        path: u.pathname || "/",
        query: u.search.slice(1),
        fragment: u.hash.slice(1),
        display: u.hostname.replace(/^www\./, ""),
      };
    } catch {
      return null;
    }
  }

  function prettyJSON(v: string): string {
    try {
      return JSON.stringify(JSON.parse(v), null, 2);
    } catch {
      return v;
    }
  }

  function jsonStats(v: string): { keys: number; depth: number; type: string } {
    try {
      const parsed = JSON.parse(v);
      const type = Array.isArray(parsed) ? "array" : typeof parsed;
      const keys =
        type === "array" ? parsed.length : Object.keys(parsed).length;
      function maxDepth(o: unknown, d = 0): number {
        if (typeof o !== "object" || o === null) return d;
        const values = Object.values(o as Record<string, unknown>);
        if (values.length === 0) return d;
        return Math.max(...values.map((v) => maxDepth(v, d + 1)));
      }
      return { keys, depth: maxDepth(parsed), type };
    } catch {
      return { keys: 0, depth: 0, type: "unknown" };
    }
  }

  function textStats(v: string) {
    const lines = v.split("\n");
    const words = v.trim().split(/\s+/).filter(Boolean).length;
    return { lines: lines.length, words, chars: v.length };
  }

  function parseEmail(v: string) {
    const [local, domain] = v.split("@");
    return { local, domain };
  }

  function getFooterMeta(
    item: (typeof listitems)[0],
    type: ContentType,
    fullText?: string | null,
  ): string[] {
    const v = fullText ?? item.name ?? "";
    switch (type) {
      case "json": {
        const s = jsonStats(v);
        return [
          s.type,
          `${s.keys} ${s.type === "array" ? "items" : "keys"}`,
          `depth ${s.depth}`,
          `${v.length} chars`,
        ];
      }
      case "multiline":
      case "text": {
        const s = textStats(v);
        const parts = [`${s.words} words`, `${s.chars} chars`];
        if (s.lines > 1) parts.unshift(`${s.lines} lines`);
        return parts;
      }
      case "url": {
        return [];
      }
      case "email": {
        const p = parseEmail(v);
        return [p.local, p.domain];
      }
      case "color":
        return [v];
      default:
        return [];
    }
  }
</script>

<div class="clipboard">
  <div class="result-list">
    {#each listitems as item, index}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="result-item"
        class:active={index === $activeIndex}
        class:pinned={item.pinned}
        data-active={index === $activeIndex}
        on:mouseenter={() => { if ($mouseHasMoved) activeIndex.set(index); }}
        on:click={() => handleClick(item)}
        on:contextmenu={(e) => {
          e.preventDefault();
          onContextMenu?.(e, item);
        }}
      >
        <div class="type-icon">
          {#if item.thumbnail}
            <img class="icon-thumb" src={item.thumbnail} alt="" />
          {:else if item.thumbnail_key}
            {#if thumbCache.has(item.thumbnail_key)}
              <img class="icon-thumb" src={thumbCache.get(item.thumbnail_key)} alt="" />
            {:else}
              <div class="icon-pill icon-text" style="font-size:0.7em">img</div>
            {/if}
          {:else if item.icon}
            <img class="icon-img" src={iconSrc(item.icon)} alt="" />
          {:else if getValidColor(item.name)}
            <div
              class="icon-swatch"
              style:background-color={getValidColor(item.name)}
            ></div>
          {:else if isURL(item.name)}
            {@const favicon = getUrlFavicon(item.name)}
            {#if favicon}
              <img
                class="icon-favicon"
                src={favicon}
                alt=""
                on:error={(e) => {
                  const img = e.target as HTMLImageElement;
                  img.style.display = "none";
                  (
                    img.nextElementSibling as HTMLElement | null
                  )?.style.setProperty("display", "flex");
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
          {:else}
            <div class="icon-pill icon-text">Aa</div>
          {/if}
        </div>
        <div class="item-body">
          <span class="item-name">{item.name}</span>
        </div>
      </div>
    {/each}
  </div>

  <div class="info-panel">
    {#if activeItem}
      <div
        class="preview-area"
        class:image-fullsize={contentType === "image"}
      >
        {#if contentType === "image"}
          <div class="image-preview-stage">
            <img
              class="image-preview"
              src={previewImage ?? activeItem.thumbnail ?? ""}
              alt={activeItem.name}
            />
          </div>
          {#if activeItem.ocr_text}
            <div class="image-ocr-text">{activeItem.ocr_text}</div>
          {/if}
        {:else if contentType === "color"}
          <div class="color-hero">
            <div class="checkerboard">
              <div
                class="main-swatch"
                style:background-color={activeColor}
              ></div>
            </div>
            <code class="color-value">{activeItem.name}</code>
          </div>
        {:else if contentType === "file"}
          {@const path = fileMetadata?.path ?? previewText ?? activeItem.name}
          <div class="file-card">
            <div class="file-heading">
              {#if activeItem.icon}
                <img class="file-preview-icon" src={iconSrc(activeItem.icon)} alt="" />
              {:else}
                <div class="file-preview-icon file-icon-fallback">FILE</div>
              {/if}
              <div class="file-name">{fileMetadata?.name ?? path.split(/[\\/]/).filter(Boolean).at(-1) ?? path}</div>
            </div>
            <table class="file-details">
              <tbody>
                <tr>
                  <th scope="row">Kind</th>
                  <td>{fileMetadata?.kind ?? "File"}</td>
                </tr>
                {#if fileMetadata?.size_bytes !== null && fileMetadata?.size_bytes !== undefined}
                  <tr>
                    <th scope="row">Size</th>
                    <td>{formatFileSize(fileMetadata.size_bytes)}</td>
                  </tr>
                {/if}
                {#if fileMetadata?.modified_unix !== null && fileMetadata?.modified_unix !== undefined}
                  <tr>
                    <th scope="row">Modified</th>
                    <td>{formatModifiedTime(fileMetadata.modified_unix)}</td>
                  </tr>
                {/if}
                <tr>
                  <th scope="row">Path</th>
                  <td class="file-path">{path}</td>
                </tr>
                <tr>
                  <th scope="row">Location</th>
                  <td class="file-path">{parentPath(path)}</td>
                </tr>
              </tbody>
            </table>
          </div>
        {:else if contentType === "url"}
          {@const parsed = parseURL(activeItem.name)}
          {@const meta = getUrlMeta(activeItem.name)}
          {@const ytId = youtubeVideoId(activeItem.name)}
          {#if parsed}
            <div class="url-card">
              <div class="url-heading">
                {#if meta?.favicon}
                  <img
                    class="url-favicon"
                    src={meta.favicon}
                    alt=""
                    on:error={(e) => {
                      (e.target as HTMLImageElement).style.display = "none";
                    }}
                  />
                {/if}
                <span class="url-hostname">{parsed.display}</span>
              </div>
              {#if meta?.thumbnail}
                <div class="url-thumb-wrap">
                  <img
                    class="url-thumb"
                    src={meta.thumbnail}
                    alt="thumbnail"
                    on:error={(e) => {
                      const img = e.target as HTMLImageElement;
                      if (img.parentElement)
                        img.parentElement.style.display = "none";
                    }}
                  />
                </div>
              {/if}
              <table class="file-details">
                <tbody>
                  <tr><th scope="row">Protocol</th><td>{parsed.protocol}</td></tr>
                  <tr><th scope="row">Domain</th><td>{parsed.hostname}</td></tr>
                  <tr><th scope="row">Path</th><td class="file-path">{parsed.path}</td></tr>
                  {#if ytId}
                    <tr><th scope="row">Platform</th><td>YouTube</td></tr>
                  {/if}
                  {#if parsed.query}
                    <tr><th scope="row">Query</th><td class="file-path">{parsed.query}</td></tr>
                  {/if}
                  {#if parsed.fragment}
                    <tr><th scope="row">Fragment</th><td class="file-path">{parsed.fragment}</td></tr>
                  {/if}
                  <tr><th scope="row">URL</th><td class="file-path">{previewText ?? activeItem.name}</td></tr>
                </tbody>
              </table>
              <!-- svelte-ignore a11y_invalid_attribute -->
              <a
                class="url-open"
                href={activeItem.name}
                target="_blank"
                rel="noopener"
              >
                Open in browser
              </a>
            </div>
          {/if}
        {:else if contentType === "email"}
          <div class="email-card">
            <div class="email-icon">✉</div>
            <div class="email-address">{activeItem.name}</div>
            <a class="url-open" href="mailto:{activeItem.name}"
              >compose email ↗</a
            >
          </div>
        {:else if contentType === "json"}
          <div class="json-container">
            <pre class="json-preview">{prettyJSON(previewText ?? activeItem.name)}</pre>
          </div>
        {:else if contentType === "multiline"}
          <div class="multiline-container">
            <div class="text-preview">{previewText ?? activeItem.name}</div>
          </div>
        {:else}
          <div class="text-container">
            <div class="text-preview">{previewText ?? activeItem.name}</div>
          </div>
        {/if}
      </div>

      <FooterBar>
        {#if contentType === "file"}
          {#if fileMetadata?.extension}
            <Chip mono>{fileMetadata.extension.toUpperCase()}</Chip>
          {/if}
        {:else}
          <Chip mono accent={TYPE_ACCENT[contentType] ?? "#DDD"}>{contentType}</Chip>
        {/if}
        {#if contentType === "image"}
          {#each imageFooterMeta as meta}
            <Chip>{meta}</Chip>
          {/each}
        {:else}
          {#each footerMeta as meta}
            <Chip>{meta}</Chip>
          {/each}
        {/if}
        <div class="footer-trailing">
          <Timestamp value={activeItem.description} />
          {#if activeItem.pinned}
            <Chip accent="rgba(200, 220, 255, 1)">Pinned</Chip>
          {/if}
        </div>
      </FooterBar>
    {/if}
  </div>
</div>

<style>
  .clipboard {
    display: flex;
    height: 100%;
    color: var(--q-font-color);
  }

  .result-list {
    flex: 0 0 224px;
    border-right: 1px solid rgba(0,0, 0, 0);
    overflow-y: auto;
    padding: 5px 10px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .result-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border-radius: var(--q-item-border-radius);
    border: 1px solid transparent;
  }

  .result-item.pinned {
    border-color: var(--q-pin-border);
  }

  .result-item.active {
    background: var(--q-active-bg-color);
    border-color: var(--q-active-border-color);
  }

  .result-item.pinned.active {
    border-color: var(--q-pin-border-active);
  }

  .footer-trailing {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-left: auto;
  }

  .footer-trailing :global(.ts) {
    margin-left: 0;
  }

  .type-icon {
    flex-shrink: 0;
    width: 28px;
    height: 28px;
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

  /* Blue tint: shift hue toward blue, boost saturation */
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
    font-size: 1.0em;
    font-family: var(--q-mono);
    padding: 2px 5px;
    border-radius: 5px;
    letter-spacing: 0.03em;
    white-space: nowrap;
    font-weight: 600;
    text-transform: uppercase;
  }
  .icon-url    { background: #1a2a3a; color: #60a5fa; border: 1px solid #2a3a4a; }
  .icon-email  { background: #2a1a3a; color: #c084fc; border: 1px solid #3a2a4a; }
  .icon-json   { background: #3a2a1a; color: #fb923c; border: 1px solid #4a3a2a; }
  .icon-text   { background: #1e1e1e; color: #888;    border: 1px solid #2e2e2e; text-transform: none; }
  .icon-img {
    width: 20px;
    height: 20px;
    object-fit: contain;
  }

  .item-body { flex: 1; min-width: 0; }

  .item-name {
    display: block;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 0.95em;
  }

  .info-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    position: relative;
  }

  .preview-area {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    /* Leave room for the image shadow so the preview area's overflow clip
       doesn't cut it off at the edges. */
    padding: 40px 40px 88px;
    box-sizing: border-box;
    overflow: hidden;
    min-height: 0;
  }

  .preview-area.image-fullsize {
    display: block;
    padding: 40px 40px 88px;
    box-sizing: border-box;
    overflow-y: auto;
  }

  .image-preview-stage {
    position: relative;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .image-preview {
    display: block;
    width: auto;
    height: auto;
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    border-radius: 8px;
    box-shadow: 0 8px 28px rgba(0, 0, 0, 0.38), 0 0 24px rgba(220, 220, 220, 0.16);
  }

  .image-ocr-text {
    margin-top: 12px;
    width: 100%;
    box-sizing: border-box;
    padding: 12px 14px;
    font-size: 1em;
    line-height: 1.55;
    color: var(--q-text-secondary);
    white-space: pre-wrap;
    word-break: break-word;
    user-select: text !important;
    -webkit-user-select: text !important;
    background: var(--q-code-bg);
    border: 1px solid var(--q-divider-dark);
    border-radius: 10px;
    opacity: 0.4;
  }

  .color-hero {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 24px;
  }

  .checkerboard {
    width: 200px;
    height: 200px;
    border-radius: 111px;
    background-image:
      linear-gradient(45deg, var(--q-surface-dark) 25%, transparent 25%),
      linear-gradient(-45deg, var(--q-surface-dark) 25%, transparent 25%),
      linear-gradient(45deg, transparent 75%, var(--q-surface-dark) 75%),
      linear-gradient(-45deg, transparent 75%, var(--q-surface-dark) 75%);
    background-size: 20px 20px;
    background-position: 0 0, 0 10px, 10px -10px, -10px 0px;
    background-color: var(--q-thumb-bg);
    overflow: hidden;
    border: 1px solid var(--q-border-dark);
  }

  .main-swatch { width: 100%; height: 100%; }

  .file-card,
  .url-card {
    width: min(100%, 560px);
    max-height: 100%;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    gap: 18px;
    overflow-y: auto;
  }

  .file-heading {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }

  .file-preview-icon {
    width: 32px;
    height: 32px;
    flex: 0 0 32px;
    box-sizing: border-box;
    object-fit: contain;
  }

  .file-icon-fallback {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--q-text-muted);
    font-size: 1em;
    font-weight: 600;
    border: 1px solid var(--q-divider-dark);
    border-radius: 5px;
  }

  .file-name {
    min-width: 0;
    overflow-wrap: anywhere;
    color: var(--q-font-color);
    font-size: 1em;
    font-weight: 600;
  }

  .file-details {
    width: 100%;
    border-collapse: collapse;
    table-layout: fixed;
    font-size: 0.9em;
  }

  .file-details tr {
    border-bottom: 1px solid var(--q-divider-dark);
  }

  .file-details th,
  .file-details td {
    padding: 9px 8px;
    text-align: left;
    vertical-align: top;
  }

  .file-details th {
    width: 110px;
    color: var(--q-text-secondary);
    font-weight: 500;
  }

  .file-details td {
    color: var(--q-text-muted);
    overflow-wrap: anywhere;
  }

  .file-details td.file-path {
    color: var(--q-text-secondary);
    font-family: var(--q-mono);
    font-size: 0.9em;
    user-select: text !important;
    -webkit-user-select: text !important;
  }

  .color-value {
    font-family: var(--q-mono);
    font-size: 1.1em;
    background: var(--q-surface-dark);
    padding: 8px 18px;
    border-radius: 12px;
    color: var(--q-font-color);
    border: 1px solid var(--q-border-dark);
  }

  .url-thumb-wrap {
    width: 100%;
    max-height: 180px;
    aspect-ratio: 16 / 7;
    overflow: hidden;
    background: var(--q-thumb-bg);
    border-radius: 8px;
    flex-shrink: 0;
  }

  .url-thumb {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .url-heading {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .url-favicon {
    width: 32px;
    height: 32px;
    object-fit: contain;
    border-radius: 6px;
    flex-shrink: 0;
  }

  .url-hostname {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--q-font-color);
    font-size: 1em;
    font-weight: 600;
  }

  .url-open {
    display: inline-flex;
    align-items: center;
    align-self: flex-start;
    border: 1px solid var(--q-divider-dark);
    border-radius: 6px;
    padding: 6px 10px;
    color: var(--q-text-secondary);
    font-size: 0.8em;
    text-decoration: none;
    transition: background-color 120ms ease, color 120ms ease;
  }
  .url-open:hover {
    background: var(--q-surface-dark);
    color: var(--q-font-color);
  }
  .url-open:focus-visible {
    outline: 2px solid var(--q-font-color);
    outline-offset: 2px;
  }

  .email-card {
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
    background: var(--q-code-bg);
    border: 1px solid var(--q-divider-dark);
    border-radius: 14px;
    padding: 28px 20px;
  }

  .email-icon { font-size: 2.2em; opacity: 0.3; }

  .email-address {
    font-size: 1em;
    font-family: var(--q-mono);
    color: #c084fc;
    word-break: break-all;
    text-align: center;
  }

  .json-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .json-preview {
    flex: 1;
    font-family: var(--q-mono);
    font-size: 0.78em;
    color: #fb923c;
    user-select: text !important;
    -webkit-user-select: text !important;
    background: var(--q-code-bg);
    border: 1px solid var(--q-divider-dark);
    border-radius: 10px;
    padding: 14px;
    overflow: auto;
    margin: 0;
    white-space: pre;
    line-height: 1.6;
  }

  .multiline-container,
  .text-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .text-preview {
    flex: 1;
    white-space: pre-wrap;
    word-break: break-word;
    user-select: text !important;
    -webkit-user-select: text !important;
    font-family: var(--q-mono);
    color: #ffb5bc;
    font-size: 0.95em;
    overflow: auto;
  }

</style>
