<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  let videoEl: HTMLVideoElement;
  let stream: MediaStream | null = null;
  let error: string | null = null;
  let mirrored = true;
  let saving = false;
  let flashVisible = false;

  onMount(async () => {
    try {
      stream = await navigator.mediaDevices.getUserMedia({ video: true });
      videoEl.srcObject = stream;
    } catch (e) {
      error = "Could not access camera";
      console.error(e);
    }
  });

  onDestroy(() => {
    stream?.getTracks().forEach((t) => t.stop());
  });

  async function capture() {
    if (saving) return;
    saving = true;

    flashVisible = true;
    setTimeout(() => (flashVisible = false), 150);

    try {
      const canvas = document.createElement("canvas");
      canvas.width = videoEl.videoWidth;
      canvas.height = videoEl.videoHeight;
      const ctx = canvas.getContext("2d")!;

      if (mirrored) {
        ctx.translate(canvas.width, 0);
        ctx.scale(-1, 1);
      }
      ctx.drawImage(videoEl, 0, 0);

      const dataUrl = canvas.toDataURL("image/png");
      const base64 = dataUrl.split(",")[1];

      const savedPath = await invoke<string>("save_capture", { pngBase64: base64 });
      console.log("Saved to", savedPath);
    } catch (e) {
      console.error("Failed to save capture:", e);
      error = "Failed to save image";
    } finally {
      saving = false;
    }
  }
</script>

<div class="camera-container">
  {#if error}
    <p class="error">{error}</p>
  {:else}
    <div class="video-wrapper">
      <video bind:this={videoEl} autoplay playsinline muted class:mirrored></video>
      {#if flashVisible}
        <div class="flash"></div>
      {/if}
    </div>
    <div class="controls">
      <button
        class="control-btn mirror-btn"
        on:click={() => (mirrored = !mirrored)}
        title="Toggle mirror"
      >
        Flip
      </button>
      <button
        class="control-btn capture-btn"
        class:saving
        on:click={capture}
        title="Capture"
        disabled={saving}
      >
        <div class="shutter-inner"></div>
      </button>
    </div>
  {/if}
</div>

<style>
  .camera-container {
    display: flex;
    align-items: stretch;
    height: 100%;
    width: 100%;
    gap: 10px;
    padding: 12px;
    box-sizing: border-box;
  }
  .video-wrapper {
    flex: 1;
    overflow: hidden;
    border-radius: 10px;
    background: #000;
    position: relative;
  }
  video {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
  video.mirrored {
    transform: scaleX(-1);
  }
  .flash {
    position: absolute;
    inset: 0;
    background: white;
    border-radius: 10px;
    opacity: 0.7;
    pointer-events: none;
    animation: flash-fade 150ms ease-out forwards;
  }
  @keyframes flash-fade {
    from { opacity: 0.7; }
    to   { opacity: 0; }
  }
  .controls {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    gap: 12px;
  }
  .control-btn {
    width: 42px;
    height: 42px;
    border-radius: 10px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.07);
    color: rgba(255, 255, 255, 0.6);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.02em;
  }
  .control-btn:active {
    transform: scale(0.92);
  }
  .control-btn:hover {
    background: rgba(255, 255, 255, 0.13);
    color: rgba(255, 255, 255, 0.95);
  }
  .capture-btn {
    width: 46px;
    height: 46px;
    border-radius: 50%;
    background: #e03030;
    border: 3px solid rgba(255, 255, 255, 0.25);
  }
  .capture-btn:hover {
    background: #c82020;
  }
  .capture-btn.saving {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .shutter-inner {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.3);
    pointer-events: none;
  }
  .error {
    color: rgba(255, 255, 255, 0.4);
    font-size: 13px;
    margin: auto;
  }
</style>
