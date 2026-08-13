<div class="bar">
  <slot />
</div>

<style>
  .bar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 14px;
    flex-shrink: 0;
    flex-wrap: wrap;
    font-size: 0.9em;
  }
</style>
