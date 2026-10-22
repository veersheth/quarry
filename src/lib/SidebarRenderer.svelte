<div class="sidebar-renderer">
  <div class="sidebar-list">
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
    flex: 0 0 224px;
    min-height: 0;
    overflow-y: auto;
  }

  .sidebar-detail {
    display: flex;
    flex: 1;
    min-width: 0;
    min-height: 0;
    flex-direction: column;
  }
</style>
