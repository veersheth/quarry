<script lang="ts">
  export let listitems: {
    name: string;
    action_id: string;
    description?: string;
    icon?: string;
  }[] = [];

  export let activeIndex: number = 0;
</script>

<div class="result-list">
  {#each listitems as item, index}
    <div class="result-item" class:active={index === activeIndex}>
      <span class="item-name">{item.name}</span>
      {#if item.description}
        <span class="item-desc">{item.description}</span>
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
    flex-direction: column;
    width: auto;
    padding: 12px 18px;
    margin: 0 12px;
    border: none;
    border-radius: 12px;
    background: none;
    text-align: left;
    color: #e0e0e0;
    cursor: pointer;
    border: 1px solid rgba(255, 255, 255, 0);
  }

  .result-item.active {
    background-color: rgba(60, 60, 60, 0.4);
  }
  .result-item:first-child.active {
    background-color: rgba(60, 60, 60, 0);
  }

  .result-item:first-child {
    padding: 1.5rem 1rem 1.0rem;
    border-radius: 0px;

    .item-name {
      font-family: Georgia, serif;
      color: rgba(235, 214, 167, 1);
      font-size: 2rem;
    }

    .item-desc {
      display: none;
    }
  }

  .item-name {
    font-size: 1.1rem;
    margin-bottom: 1rem;
  }

  .item-desc {
    opacity: 0.4;
    font-size: 16px;
  }
</style>
