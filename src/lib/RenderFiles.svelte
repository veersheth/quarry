<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { writable, type Writable } from "svelte/store";
  import type { ResultItem } from "../stores/search";
  import { mouseHasMoved } from "../stores/search";
  import SidebarRenderer from "./SidebarRenderer.svelte";
  import ItemIcon from "./ui/ItemIcon.svelte";
  import FooterBar from "./ui/FooterBar.svelte";
  import Chip from "./ui/Chip.svelte";

  export let listitems: ResultItem[] = [];
  export let activeIndex: Writable<number> = writable(0);
  export let onContextMenu:
    | ((event: MouseEvent, item: ResultItem) => void)
    | undefined = undefined;

  type FileMetadata = {
    name: string;
    path: string;
    kind: string;
    extension: string | null;
    size_bytes: number | null;
    modified_unix: number | null;
    folder_child_count: number | null;
    folder_entries: string[] | null;
    image_width: number | null;
    image_height: number | null;
  };

  type FilePreview = {
    text: string | null;
    image_data: string | null;
    truncated: boolean;
  };

  const IMAGE_EXTENSIONS = new Set(["png", "jpg", "jpeg", "gif", "webp", "bmp", "tif", "tiff"]);
  const VIDEO_EXTENSIONS = new Set(["mp4", "mkv", "mov", "webm", "avi", "m4v", "mpeg", "mpg"]);
  const AUDIO_EXTENSIONS = new Set(["mp3", "wav", "flac", "ogg", "opus", "m4a", "aac"]);
  const CODE_EXTENSIONS = new Set([
    "rs", "js", "jsx", "ts", "tsx", "svelte", "vue", "py", "sh", "bash", "zsh",
    "fish", "sql", "go", "java", "kt", "c", "h", "cpp", "hpp", "cs", "rb", "php",
    "swift", "lua", "css", "scss", "html", "htm", "json", "jsonc", "toml", "yaml",
    "yml", "xml", "nix", "graphql", "gql", "proto", "diff", "dockerfile", "makefile",
  ]);

  $: activeItem = listitems[$activeIndex];
  $: activePath = activeItem?.description ?? activeItem?.draggable_path ?? "";
  $: extension = activePath.split(".").pop()?.toLowerCase() ?? "";
  $: category = getCategory(activeItem, extension);
  $: previewLines = preview?.text?.split("\n").length ?? 0;

  let metadata: FileMetadata | null = null;
  let preview: FilePreview | null = null;
  let requestSequence = 0;
  let detailsLoaded = false;

  $: {
    listitems;
    mouseHasMoved.set(false);
  }

  $: if (activePath) void loadDetails(activePath);
  $: if (!activePath) {
    requestSequence++;
    metadata = null;
    preview = null;
    detailsLoaded = false;
  }

  async function loadDetails(path: string) {
    const request = ++requestSequence;
    metadata = null;
    preview = null;
    detailsLoaded = false;
    try {
      const details = await invoke<FileMetadata | null>("get_clipboard_file_metadata", { path });
      if (request !== requestSequence) return;
      metadata = details;
      detailsLoaded = true;
      if (!details || details.kind === "Folder") return;

      const content = await invoke<FilePreview | null>("get_file_preview", { path });
      if (request === requestSequence) preview = content;
    } catch (error) {
      if (request === requestSequence) {
        detailsLoaded = true;
        console.warn("Could not load file preview:", error);
      }
    }
  }

  function getCategory(item: ResultItem | undefined, ext: string): string {
    if (item?.icon?.includes("folder")) return "Folder";
    if (IMAGE_EXTENSIONS.has(ext)) return "Image";
    if (VIDEO_EXTENSIONS.has(ext)) return "Video";
    if (AUDIO_EXTENSIONS.has(ext)) return "Audio";
    if (CODE_EXTENSIONS.has(ext)) return ext === "sql" ? "SQL" : "Code";
    if (["txt", "md", "markdown", "rst", "log", "csv", "tsv"].includes(ext)) return "Text";
    if (ext === "pdf") return "PDF document";
    return "File";
  }

  function formatSize(bytes: number | null | undefined): string {
    if (bytes == null) return "—";
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 ** 2) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 ** 3) return `${(bytes / 1024 ** 2).toFixed(1)} MB`;
    return `${(bytes / 1024 ** 3).toFixed(1)} GB`;
  }

  function formatDate(timestamp: number | null | undefined): string {
    return timestamp == null ? "—" : new Date(timestamp * 1000).toLocaleString();
  }
</script>

<SidebarRenderer>
  <div slot="list" class="file-list">
    {#each listitems as item, index (item.name + index)}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div
        class="file-row"
        class:active={index === $activeIndex}
        data-active={index === $activeIndex}
        on:mouseenter={() => {
          if ($mouseHasMoved) activeIndex.set(index);
        }}
        on:click={() => activeIndex.set(index)}
        on:contextmenu={(event) => {
          event.preventDefault();
          onContextMenu?.(event, item);
        }}
      >
        <ItemIcon icon={item.icon ?? "icons/file.svg"} name={item.name} size={22} />
        <span class="file-name">{item.name}</span>
      </div>
    {:else}
      <div class="empty-list">No matching files</div>
    {/each}
  </div>

  <div slot="detail" class="file-detail">
    {#if activeItem}
      <div class="detail-scroll">
        <header class="file-heading">
          <ItemIcon icon={activeItem.icon ?? "icons/file.svg"} name={activeItem.name} size={42} />
          <div class="heading-text">
            <h2>{activeItem.name}</h2>
            <span>{metadata?.kind ?? category}{metadata?.extension ? ` · ${metadata.extension.toUpperCase()}` : ""}</span>
          </div>
        </header>

        {#if category === "Image" && preview?.image_data}
          <div class="image-preview">
            <img src={preview.image_data} alt={`Preview of ${activeItem.name}`} />
          </div>
        {:else if category === "Code" || category === "SQL" || category === "Text"}
          {#if preview?.text != null}
            <div class="preview-label">
              <span>{category === "Text" ? "TEXT PREVIEW" : `${extension.toUpperCase()} PREVIEW`}</span>
              <span>{previewLines.toLocaleString()} lines</span>
            </div>
            <pre class="text-preview">{preview.text}</pre>
            {#if preview.truncated}<div class="preview-note">Preview limited to the first 32 KB</div>{/if}
          {:else}
            <div class="type-card"><span class="type-glyph">Aa</span><span>Text preview unavailable</span></div>
          {/if}
        {:else if category === "Folder"}
          <div class="folder-summary">
            <strong>{metadata?.folder_child_count === 101 ? "100+" : (metadata?.folder_child_count ?? 0).toLocaleString()}</strong>
            <span>items in this folder</span>
          </div>
          {#if metadata?.folder_entries?.length}
            <div class="folder-contents">
              <div class="section-label">CONTENTS</div>
              {#each metadata.folder_entries as entry (entry)}
                <div class="folder-entry"><span>›</span>{entry}</div>
              {/each}
            </div>
          {/if}
        {:else if category === "Video" || category === "Audio" || category === "PDF document"}
          <div class="type-card">
            <span class="type-glyph">{category === "Video" ? "▶" : category === "Audio" ? "♫" : "PDF"}</span>
            <span>{category}{metadata?.extension ? ` · ${metadata.extension.toUpperCase()}` : ""}</span>
          </div>
        {:else}
          <div class="type-card"><span class="type-glyph">···</span><span>File details</span></div>
        {/if}

        {#if metadata}
          <div class="metadata-table">
            <div class="metadata-row"><span>TYPE</span><span>{category}</span></div>
            {#if metadata.extension}
              <div class="metadata-row"><span>FORMAT</span><span>{metadata.extension.toUpperCase()}</span></div>
            {/if}
            {#if metadata.image_width && metadata.image_height}
              <div class="metadata-row"><span>DIMENSIONS</span><span>{metadata.image_width} × {metadata.image_height} px</span></div>
            {/if}
            {#if metadata.size_bytes != null}
              <div class="metadata-row"><span>SIZE</span><span>{formatSize(metadata.size_bytes)}</span></div>
            {/if}
            <div class="metadata-row"><span>MODIFIED</span><span>{formatDate(metadata.modified_unix)}</span></div>
            <div class="metadata-row"><span>LOCATION</span><span>{activePath.slice(0, Math.max(activePath.lastIndexOf("/"), 1))}</span></div>
            <div class="metadata-row path-row"><span>PATH</span><span>{activePath}</span></div>
          </div>
        {:else if !detailsLoaded}
          <div class="detail-loading">Loading file details…</div>
        {:else}
          <div class="detail-loading">This file is unavailable.</div>
        {/if}
      </div>

      <FooterBar>
        <Chip mono accent="#60a5fa">{category.toLowerCase()}</Chip>
        {#if metadata?.size_bytes != null}<Chip>{formatSize(metadata.size_bytes)}</Chip>{/if}
        {#if metadata?.image_width && metadata.image_height}<Chip>{metadata.image_width} × {metadata.image_height}</Chip>{/if}
      </FooterBar>
    {:else}
      <div class="no-selection">Select a file to inspect it</div>
    {/if}
  </div>
</SidebarRenderer>

<style>
  .file-list {
    display: flex;
    flex-direction: column;
    gap: 5px;
    padding: 5px 10px;
  }

  .file-row {
    display: flex;
    min-width: 0;
    align-items: center;
    gap: 10px;
    padding: 9px 10px;
    border: 1px solid transparent;
    border-radius: var(--q-item-border-radius);
  }

  .file-row.active {
    border-color: var(--q-active-border-color);
    background: var(--q-active-bg-color);
  }

  .file-name {
    overflow: hidden;
    font-size: 0.9em;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .empty-list,
  .no-selection,
  .detail-loading {
    margin: auto;
    color: var(--q-text-dim);
  }

  .file-detail {
    display: flex;
    min-width: 0;
    min-height: 0;
    flex: 1;
    flex-direction: column;
  }

  .detail-scroll {
    min-width: 0;
    min-height: 0;
    flex: 1;
    overflow: auto;
    padding: 24px 28px 16px;
  }

  .file-heading {
    display: flex;
    align-items: center;
    gap: 14px;
    margin-bottom: 20px;
  }

  .heading-text { min-width: 0; }

  h2 {
    overflow: hidden;
    margin: 0 0 4px;
    font-size: 1.05em;
    font-weight: 550;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .heading-text span {
    color: var(--q-text-dim);
    font-size: 0.8em;
  }

  .image-preview {
    display: flex;
    height: min(42vh, 300px);
    align-items: center;
    justify-content: center;
    margin-bottom: 20px;
    overflow: hidden;
    border: 1px solid var(--q-border-subtle);
    border-radius: 12px;
    background: rgba(0, 0, 0, 0.18);
  }

  .image-preview img {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  .preview-label,
  .metadata-row {
    display: grid;
    grid-template-columns: minmax(96px, 0.32fr) minmax(0, 1fr);
    gap: 14px;
  }

  .preview-label {
    margin-bottom: 7px;
    color: var(--q-text-dim);
    font: 0.7em var(--q-mono);
  }

  .text-preview {
    max-height: 280px;
    margin: 0 0 7px;
    padding: 14px 16px;
    overflow: auto;
    border: 1px solid var(--q-border-subtle);
    border-radius: 10px;
    background: rgba(0, 0, 0, 0.2);
    color: var(--q-text-secondary);
    font: 0.78em/1.55 var(--q-mono);
    tab-size: 2;
    white-space: pre;
  }

  .preview-note {
    margin-bottom: 18px;
    color: var(--q-text-dim);
    font-size: 0.72em;
  }

  .folder-summary {
    display: flex;
    align-items: baseline;
    gap: 8px;
    margin: 22px 0;
  }

  .folder-summary strong {
    font-size: 2em;
    font-weight: 500;
  }

  .folder-summary span,
  .section-label { color: var(--q-text-dim); }

  .section-label {
    margin-bottom: 7px;
    font: 0.7em var(--q-mono);
  }

  .folder-contents {
    margin-bottom: 22px;
  }

  .folder-entry {
    display: flex;
    gap: 9px;
    padding: 5px 0;
    overflow: hidden;
    color: var(--q-text-secondary);
    font-size: 0.84em;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .folder-entry span { color: var(--q-text-dim); }

  .type-card {
    display: flex;
    min-height: 120px;
    align-items: center;
    justify-content: center;
    gap: 14px;
    margin-bottom: 20px;
    border: 1px solid var(--q-border-subtle);
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.025);
    color: var(--q-text-dim);
  }

  .type-glyph {
    color: var(--q-text-secondary);
    font-size: 1.6em;
  }

  .metadata-table {
    overflow: hidden;
    border: 1px solid var(--q-border-subtle);
    border-radius: 10px;
  }

  .metadata-row {
    min-height: 38px;
    align-items: center;
    padding: 0 12px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.045);
    font-size: 0.78em;
  }

  .metadata-row:last-child { border-bottom: 0; }

  .metadata-row > :first-child {
    color: var(--q-text-dim);
    font: 0.9em var(--q-mono);
    letter-spacing: 0.035em;
  }

  .metadata-row > :last-child {
    overflow-wrap: anywhere;
    color: var(--q-text-secondary);
  }

  .path-row { padding-top: 8px; padding-bottom: 8px; }
</style>
