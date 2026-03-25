<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  let videoEl: HTMLVideoElement;
  let stream: MediaStream | null = null;
  let error: string | null = null;
  let mirrored = true;
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
  function capture() {
    alert("placeholder");
  }
</script>

<div class="camera-container">
  {#if error}
    <p class="error">{error}</p>
  {:else}
    <div class="video-wrapper">
      <video bind:this={videoEl} autoplay playsinline muted class:mirrored
      ></video>
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
        on:click={capture}
        title="Capture"
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
    background: rgba(20, 20, 20, 1);
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
