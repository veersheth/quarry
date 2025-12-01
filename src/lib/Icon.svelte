<script lang="ts">
  export let icon: string;
</script>

{#if icon && (icon.startsWith("data:image") || icon.includes(".png") || icon.includes(".svg"))}
  <img class="item-icon" src={icon} alt="" />
{:else if icon}
  <span class="item-icon">{icon}</span>
{:else}
  <div class="item-icon fallback"></div>
{/if}

<style>
  img.item-icon, .item-icon.fallback {
    width: 20px;
    height: 20px;
  }

  span.item-icon {
    font-size: 2rem; 
    line-height: 1;
    margin-bottom: 6px;
  }

  .item-icon.fallback {
    border: 1px solid rgba(60, 60, 60, 1);
    border-radius: 30%;
  }
</style>

