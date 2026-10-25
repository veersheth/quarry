<script lang="ts">
  import type { MetadataRow } from "./metadata";

  export let rows: MetadataRow[] = [];
</script>

<table class="metadata-table">
  <tbody>
    {#each rows as row (row.label)}
      <tr>
        <th scope="row">{row.label}</th>
        <td class:mono={row.mono}>{row.value}</td>
      </tr>
    {/each}
  </tbody>
</table>

<style>
  .metadata-table {
    width: 100%;
    margin-top: 5px;
    border: 1px solid var(--q-table-border, rgba(255, 255, 255, 0.07));
    border-radius: 9px;
    border-spacing: 0;
    overflow: hidden;
    background: var(--q-table-surface, rgba(0, 0, 0, 0.12));
    box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.18);
    table-layout: fixed;
    font-size: 0.94em;
  }

  tr:not(:last-child) th,
  tr:not(:last-child) td {
    border-bottom: 1px solid var(--q-table-divider, rgba(255, 255, 255, 0.055));
  }

  th,
  td {
    padding: 10px 11px;
    text-align: left;
    vertical-align: top;
    overflow-wrap: anywhere;
  }

  th {
    width: 106px;
    background: var(--q-table-label-surface, rgba(255, 255, 255, 0.018));
    color: var(--q-text-secondary, rgba(255, 255, 255, 0.8));
    font-size: 0.95em;
    font-weight: 500;
  }

  td {
    color: var(--q-text-muted, rgba(255, 255, 255, 0.7));
    user-select: text !important;
    -webkit-user-select: text !important;
  }

  td.mono {
    color: var(--q-text-secondary, rgba(255, 255, 255, 0.8));
    font-family: var(--q-mono, monospace);
    font-size: 0.98em;
  }
</style>
