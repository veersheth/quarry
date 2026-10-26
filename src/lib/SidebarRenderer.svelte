<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  export let searcherKey: "clipboard" | "files" | "fonts";
  export let listWidth = 224;

  let renderer: HTMLDivElement;
  let sidebarList: HTMLDivElement;
  let width = listWidth;
  let isDragging = false;
  let hasUserResized = false;

  onMount(async () => {
    try {
      const savedWidth = await invoke<number>("get_sidebar_width", { searcher: searcherKey });
      if (!hasUserResized) width = clampWidth(savedWidth);
    } catch (error) {
      console.warn(`Could not load ${searcherKey} sidebar width:`, error);
    }
  });

  function maxWidth(): number {
    if (!renderer) return 560;
    return Math.max(160, Math.min(560, renderer.clientWidth - 276));
  }

  function clampWidth(value: number): number {
    return Math.min(maxWidth(), Math.max(160, value));
  }

  function beginResize(event: PointerEvent) {
    if (event.button !== 0) return;
    isDragging = true;
    event.currentTarget instanceof HTMLElement && event.currentTarget.setPointerCapture(event.pointerId);
    event.preventDefault();
  }

  function resize(event: PointerEvent) {
    if (!isDragging || !sidebarList) return;
    const bounds = sidebarList.getBoundingClientRect();
    width = clampWidth(event.clientX - bounds.left - 4);
  }

  function finishResize() {
    if (!isDragging) return;
    isDragging = false;
    hasUserResized = true;
    void saveWidth();
  }

  function adjustWithKeyboard(event: KeyboardEvent) {
    let nextWidth: number | undefined;
    if (event.key === "ArrowLeft") nextWidth = width - 16;
    if (event.key === "ArrowRight") nextWidth = width + 16;
    if (event.key === "Home") nextWidth = 160;
    if (event.key === "End") nextWidth = maxWidth();
    if (nextWidth === undefined) return;
    event.preventDefault();
    width = clampWidth(nextWidth);
    hasUserResized = true;
    void saveWidth();
  }

  async function saveWidth() {
    try {
      await invoke("save_sidebar_width", { searcher: searcherKey, width: Math.round(width) });
    } catch (error) {
      console.warn(`Could not save ${searcherKey} sidebar width:`, error);
    }
  }
</script>

<div bind:this={renderer} class="sidebar-renderer" class:resizing={isDragging}>
  <div bind:this={sidebarList} class="sidebar-list" style={`--sidebar-list-width: ${width}px`}>
    <slot name="list" />
  </div>
  <div
    class="sidebar-resizer"
    role="separator"
    aria-label={`${searcherKey} sidebar width`}
    aria-orientation="vertical"
    aria-valuemin="160"
    aria-valuemax={maxWidth()}
    aria-valuenow={Math.round(width)}
    tabindex="0"
    on:pointerdown={beginResize}
    on:pointermove={resize}
    on:pointerup={finishResize}
    on:pointercancel={finishResize}
    on:keydown={adjustWithKeyboard}
  ></div>
  <div class="sidebar-detail">
    <slot name="detail" />
  </div>
</div>

<style>
  .sidebar-renderer {
    display: flex;
    width: 100%;
    height: 100%;
    min-height: 0;
    box-sizing: border-box;
    gap: 0;
    padding: 12px;
    color: var(--q-font-color);
  }

  .sidebar-list {
    flex: 0 0 var(--sidebar-list-width, 224px);
    min-width: 160px;
    min-height: 0;
    overflow-y: auto;
    box-sizing: border-box;
    border: 1px solid transparent;
    border-radius: 14px;
    background: var(--q-surface-background, #242424);
    scrollbar-gutter: stable;
    scrollbar-width: thin;
    scrollbar-color: rgba(255, 255, 255, 0.12) transparent;
  }

  .sidebar-list::-webkit-scrollbar {
    width: 8px;
  }

  .sidebar-list::-webkit-scrollbar-track {
    margin-block: 8px;
    background: transparent;
  }

  .sidebar-list::-webkit-scrollbar-thumb {
    border: 2px solid transparent;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.12);
    background-clip: padding-box;
  }

  .sidebar-list::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.22);
    background-clip: padding-box;
  }

  .sidebar-resizer {
    flex: 0 0 12px;
    cursor: col-resize;
    touch-action: none;
    outline: none;
  }

  .sidebar-resizer:focus-visible {
    border-radius: 4px;
    outline: 1px solid var(--q-border-strong, rgba(255, 255, 255, 0.28));
    outline-offset: -2px;
  }

  .resizing {
    cursor: col-resize;
    user-select: none;
  }

  .sidebar-detail {
    display: flex;
    flex: 1;
    min-width: 0;
    min-height: 0;
    flex-direction: column;
  }
</style>
