<script lang="ts">
  import type { ResultItem } from "../stores/search";
  import { runItemAction } from "./keyHandler";
  export let listitems: ResultItem[] = [];
  function find(name: string): ResultItem | undefined {
    return listitems.find((i) => i.name === name);
  }
  const titleItem = () => find("Title");
  const artistItem = () => find("Artist");
  const albumArt = () => find("AlbumArt");
  const playPause = () => find("PlayPause");
  const next = () => find("Next");
  const previous = () => find("Previous");
  function click(item?: ResultItem) {
    if (item) runItemAction(item);
  }
  const isPlaying = () => playPause()?.name === "Pause";
</script>
<div class="player">
  <div class="media">
    <div
      class="art"
      style="background-image: url('{albumArt()?.description ?? ''}')"
    ></div>
    <div class="info">
      <div class="title">{titleItem()?.description ?? "—"}</div>
      <div class="artist">{artistItem()?.description ?? " "}</div>
    </div>
  </div>
  <div class="controls">
    <button
      class="small"
      disabled={!previous()}
      on:click={() => click(previous())}>◀</button
    >
    <button
      class="big"
      disabled={!playPause()}
      on:click={() => click(playPause())}
    >
      {isPlaying() ? "⏸" : "▶"}
    </button>
    <button class="small" disabled={!next()} on:click={() => click(next())}
      >▶</button
    >
  </div>
</div>
<style>
  .player {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    height: 100%;
  }
  .media {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .art {
    position: absolute;
    bottom: 0;
    right: 0;
    width: 20rem;
    height: 20rem;
    margin: 7rem;
    flex-shrink: 0;
    border-radius: 50%;
    z-index: -1;
    background-size: cover;
    background-position: center;
    filter: blur(40px);
    will-change: auto;
  }
  .info {
    min-width: 0;
    text-align: left;
    flex: 1;
  }
  .title,
  .artist {
    height: 1.4em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .artist {
    opacity: 0.6;
  }
  .controls {
    display: flex;
    gap: 8px;
  }
  button {
    border: 1px solid rgba(255, 255, 255, 0.2);
    padding: 2rem;
    color: inherit;
    background: rgba(20, 20, 20, 0.2);
    cursor: pointer;
    transition: transform 200ms ease;
  }
  .big {
    border-radius: 30px;
  }
  .small {
    border-radius: 50%;
  }
  button:hover:not(:disabled) {
    transform: scale(1.05);
  }
  button:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }
</style>
