<script lang="ts">
  export let listitems: {
    name: string;        // description for display below emoji
    exec: string;
    description?: string; // optional, can show more info if desired
    icon?: string;       // the emoji itself
  }[] = [];
  export let activeIndex: number = 0;

  function truncate(str: string | undefined, maxLength: number): string {
    if (!str) return "";
    return str.length > maxLength ? str.slice(0, maxLength) + "…" : str;
  }
</script>

<div class="result-grid">
  {#each listitems as item, index}
    <div class="grid-item" class:active={index === activeIndex}>
      {#if item.icon}
        <span class="item-icon">{item.icon}</span>
      {/if}
      <span class="item-name">{truncate(item.name, 20)}</span>
    </div>
  {/each}
</div>

<style>
  .result-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(80px, 1fr));
    gap: 12px;
    padding: 10px;
  }

  .grid-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 16px;
    border-radius: 12px;
    border: 1px solid rgba(60, 60, 60, 0.7);
    background-color: rgba(30, 30, 30, 0.5);
    cursor: pointer;
    /* transition: background 0.1s; */
    text-align: center;
  }

  .grid-item.active {
    background-color: rgba(60, 60, 60, 0.7);
    box-shadow: 0 0 20px rgba(50, 50, 50, 1);
    border: 1px solid rgba(100, 100, 100, 0.7);
  }

  .item-icon {
    font-size: 2rem; 
    line-height: 1;
    margin-bottom: 6px;
  }

  .item-name {
    font-size: 0.9rem;
    color: #e0e0e0;
    word-break: break-word;
  }
</style>

