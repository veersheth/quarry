<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { writable, type Writable } from "svelte/store";
  import type { ResultItem } from "../stores/search";
  import { mouseHasMoved } from "../stores/search";
  import SidebarRenderer from "./SidebarRenderer.svelte";
  import PreviewCard from "./ui/PreviewCard.svelte";
  import MetadataTable from "./ui/MetadataTable.svelte";
  import type { MetadataRow } from "./ui/metadata";
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
    image_width: number | null;
    image_height: number | null;
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
  $: metadataRows = getMetadataRows(metadata, category, activePath);

  let metadata: FileMetadata | null = null;
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
    detailsLoaded = false;
  }

  async function loadDetails(path: string) {
    const request = ++requestSequence;
    metadata = null;
    detailsLoaded = false;
    try {
      const details = await invoke<FileMetadata | null>("get_clipboard_file_metadata", { path });
      if (request !== requestSequence) return;
      metadata = details;
      detailsLoaded = true;
    } catch (error) {
      if (request === requestSequence) {
        detailsLoaded = true;
        console.warn("Could not load file details:", error);
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

  function getMetadataRows(details: FileMetadata | null, type: string, path: string): MetadataRow[] {
    if (!details) return [];
    const rows: MetadataRow[] = [{ label: "Kind", value: type }];
    if (details.extension) rows.push({ label: "Format", value: details.extension.toUpperCase() });
    if (details.folder_child_count != null) {
      const count = details.folder_child_count === 101 ? "100+" : details.folder_child_count.toLocaleString();
      rows.push({ label: "Items", value: `${count} in folder` });
    }
    if (details.image_width && details.image_height) {
      rows.push({ label: "Dimensions", value: `${details.image_width} × ${details.image_height} px` });
    }
    if (details.size_bytes != null) rows.push({ label: "Size", value: formatSize(details.size_bytes) });
    rows.push({ label: "Modified", value: formatDate(details.modified_unix) });
    rows.push({ label: "Location", value: path.slice(0, Math.max(path.lastIndexOf("/"), 1)), mono: true });
    rows.push({ label: "Path", value: path, mono: true });
    return rows;
  }
</script>

<SidebarRenderer searcherKey="files" listWidth={280}>
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
        <PreviewCard
          icon={activeItem.icon}
          title={metadata?.name ?? activeItem.name}
          subtitle={`${category}${metadata?.extension ? ` · ${metadata.extension.toUpperCase()}` : ""}`}
        >
          {#if metadataRows.length}
            <MetadataTable rows={metadataRows} />
          {:else if !detailsLoaded}
            <div class="detail-loading">Loading file details…</div>
          {:else}
            <div class="detail-loading">This file is unavailable.</div>
          {/if}
        </PreviewCard>
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
    gap: 4px;
    padding: 5px 4px;
  }

  .file-row {
    display: flex;
    min-width: 0;
    align-items: center;
    gap: 10px;
    padding: 9px 6px;
    border: 1px solid transparent;
    border-radius: 9px;
  }

  .file-row.active {
    border-color: transparent;
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
    display: flex;
    min-width: 0;
    min-height: 0;
    flex: 1;
    align-items: flex-start;
    overflow: auto;
    padding: 0;
    box-sizing: border-box;
  }

  :global(.file-detail .preview-card-container) {
    width: 100%;
    min-width: 0;
  }

  :global(.file-detail .preview-card) {
    max-height: none;
    overflow: visible;
  }

</style>
