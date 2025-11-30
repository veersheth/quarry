<script lang="ts">
  export let listitems: {
    name: string;
    exec: string;
    description?: string;
    icon?: string;
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
        <img class="item-icon" src={item.icon} alt="" />
      {/if}
      <span class="item-name">{truncate(item.name, 20)}</span>
    </div>
  {/each}
</div>

<style>
  .result-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
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
    background-color: rgba(30, 30, 30, 0.5);
    cursor: pointer;
    transition: background 0.2s;
    text-align: center;
  }

  .grid-item.active {
    background-color: rgba(60, 60, 60, 0.7);
  }

  .item-icon {
    width: 40px;
    height: 40px;
    margin-bottom: 8px;
    object-fit: contain;
  }

  .item-name {
    font-size: 14px;
    color: #e0e0e0;
    word-break: break-word;
  }
</style>

