<script lang="ts">
  import { onMount } from "svelte";
  import QRCode from "qrcode";
  import type { ResultItem } from "../stores/search";
  import { addToast } from "../stores/toasts";

  export let listitems: ResultItem[] = [];

  $: text = listitems[0]?.name ?? "";

  let svgData = "";
  let error = "";

  $: if (text) {
    QRCode.toString(text, { type: "svg", margin: 2, width: 240, color: { dark: "#ffffff", light: "#00000000" } })
      .then((svg) => { svgData = svg; error = ""; })
      .catch(() => { svgData = ""; error = "Text too long for QR code"; });
  }

  async function copyImage() {
    if (!text) return;
    try {
      // Render a black-on-white QR for the clipboard (visible on any background)
      const size = 512;
      const svgStr = await QRCode.toString(text, {
        type: "svg", margin: 3, width: size,
        color: { dark: "#000000", light: "#ffffff" },
      });
      const blob = new Blob([svgStr], { type: "image/svg+xml" });
      const url = URL.createObjectURL(blob);
      const img = new Image();
      img.src = url;
      await new Promise<void>((res, rej) => { img.onload = () => res(); img.onerror = rej; });
      const canvas = document.createElement("canvas");
      canvas.width = size;
      canvas.height = size;
      canvas.getContext("2d")!.drawImage(img, 0, 0, size, size);
      URL.revokeObjectURL(url);
      const png = await new Promise<Blob>((res, rej) =>
        canvas.toBlob((b) => b ? res(b) : rej(new Error("toBlob failed")), "image/png")
      );
      await navigator.clipboard.write([new ClipboardItem({ "image/png": png })]);
      addToast("QR image copied");
    } catch {
      await navigator.clipboard.writeText(text);
    }
  }

  onMount(() => {
    function onKeydown(e: KeyboardEvent) {
      if (e.key === "Enter") { e.preventDefault(); copyImage(); }
    }
    window.addEventListener("keydown", onKeydown);
    return () => window.removeEventListener("keydown", onKeydown);
  });
</script>

<div class="qr-wrap">
  {#if error}
    <p class="qr-error">{error}</p>
  {:else if svgData}
    <div class="qr-box">
      <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
      <div class="qr-svg" on:click={copyImage}>
        {@html svgData}
      </div>
      <p class="qr-label">{text}</p>
    </div>
  {/if}
</div>

<style>
  .qr-wrap {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    padding: 20px 20px 0;
    gap: 0;
  }

  .qr-box {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
  }

  .qr-svg {
    width: 200px;
    height: 200px;
    cursor: pointer;
    border-radius: var(--q-item-border-radius, 10px);
    padding: 10px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid var(--q-border-medium);
    transition: background 0.15s ease;
  }

  .qr-svg:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .qr-svg :global(svg) {
    width: 100%;
    height: 100%;
    display: block;
  }

  .qr-label {
    font-size: 0.82em;
    color: var(--q-text-secondary);
    font-family: var(--q-mono);
    max-width: 320px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    text-align: center;
    margin: 0;
  }

  .qr-error {
    color: var(--q-text-muted);
    font-size: 0.9em;
    margin: 0;
  }
</style>
