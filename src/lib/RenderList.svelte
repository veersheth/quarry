<script lang="ts">
    import Icon from "./Icon.svelte";

  export let listitems: {
    name: string;
    action_id: string;
    description?: string;
    icon?: string;
  }[] = [];
  export let activeIndex: number = 0;

  function truncate(str: string | undefined, maxLength: number): string {
    if (!str) return "";
    return str.length > maxLength ? str.slice(0, maxLength) + "…" : str;
  }
</script>

<div class="result-list">
  {#each listitems as item, index}
    <div class="result-item" class:active={index === activeIndex}>
      <Icon icon={item.icon || ""} />
      <span class="item-name">{item.name}</span>
      {#if item.description}
        <span class="item-desc">{truncate(item.description, 70)}</span>
      {/if}
    </div> 
  {/each}
</div>

<style>
  .result-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 5px 0;
  }

  .result-item {
    display: flex;
    width: auto;
    padding: 12px 18px;
    margin: 0 12px;
    border: none;
    border-radius: 12px;
    background: none;
    text-align: left;
    color: #e0e0e0;
    cursor: pointer;
    border: 1px solid rgba(0, 0, 0, 0);
  }

  .result-item.active {
    background-color: rgba(60, 60, 60, 0.7);
    /* border: 1px solid rgba(120, 120, 120, 0.7); */
  }

  .item-name {
    margin: auto 0.7rem;
  }
  .item-desc {
    opacity: 0.4;
    font-size: 16px;
  }
</style>
