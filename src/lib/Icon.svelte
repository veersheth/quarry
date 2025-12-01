<script lang="ts">
  export let icon: string;
</script>

{#if icon && (icon.startsWith("data:image") || icon.includes(".png") || icon.includes(".svg"))}
  <img class="item-icon" src={icon} alt="" />
{:else}
  <span class="item-icon">{icon}</span>
{/if}

<style>
  img.item-icon {
    width: 20px;
    height: 20px;
  }

  span.item-icon {
    font-size: 2rem; 
    line-height: 1;
    margin-bottom: 6px;
  }

</style>
