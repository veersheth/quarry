<script lang="ts">
  import { onMount, onDestroy } from "svelte";

  let videoEl: HTMLVideoElement;
  let stream: MediaStream | null = null;
  let error: string | null = null;

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
</script>

<div class="camera-container">
  {#if error}
    <p class="error">{error}</p>
  {:else}
    <video bind:this={videoEl} autoplay playsinline muted />
  {/if}
</div>

<style>
  .camera-container {
    display: flex;
    justify-content: center;
    align-items: center;
    padding: 16px;
  }

  video {
    width: 100%;
    max-height: 400px;
    border-radius: 10px;
    object-fit: cover;
  }

  .error {
    color: rgba(255, 255, 255, 0.4);
    font-size: 13px;
  }
</style>
