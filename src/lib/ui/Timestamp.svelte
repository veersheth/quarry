<script lang="ts">
  /** Unix epoch seconds (number or string) → relative time.
   *  Any non-numeric string (e.g. "2024-01-15 14:30") is shown as-is. */
  export let value: string | number | undefined = undefined;

  function format(v: string | number | undefined): string {
    if (v === undefined || v === "" || v === null) return "";
    const ts = typeof v === "string" ? Number(v) : v;
    if (isNaN(ts)) return String(v);
    const age = Date.now() / 1000 - ts;
    if (age < 60)    return "just now";
    if (age < 3600)  return `${Math.floor(age / 60)}m ago`;
    if (age < 86400) return `${Math.floor(age / 3600)}h ago`;
    return `${Math.floor(age / 86400)}d ago`;
  }

  $: display = format(value);
</script>

{#if display}
  <span class="ts">{display}</span>
{/if}

<style>
  .ts {
    margin-left: auto;
    opacity: 0.35;
    white-space: nowrap;
  }
</style>
