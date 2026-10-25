<script lang="ts">
  export let listWidth = 224;
</script>

<div class="sidebar-renderer">
  <div class="sidebar-list" style={`--sidebar-list-width: ${listWidth}px`}>
    <slot name="list" />
  </div>
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
    color: var(--q-font-color);
  }

  .sidebar-list {
    flex: 0 0 min(var(--sidebar-list-width, 224px), 40vw);
    min-height: 0;
    overflow-y: auto;
    border-right: 1px solid var(--q-divider-dark, rgba(255, 255, 255, 0.08));
    background: color-mix(in srgb, var(--q-bg-color, #101010) 97%, white 3%);
  }

  .sidebar-detail {
    display: flex;
    flex: 1;
    min-width: 0;
    min-height: 0;
    flex-direction: column;
  }
</style>
