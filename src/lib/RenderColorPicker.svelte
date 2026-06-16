<script>
  import { addToast } from "../stores/toasts";

  export let initialColor = "";

  let hue = 0;
  let saturation = 100;
  let lightness = 50;
  let isDraggingSL = false;
  let isDraggingHue = false;

  function hexToRgb(hex) {
    let h = hex.replace("#", "");
    if (h.length === 3 || h.length === 4)
      h = h
        .split("")
        .map((c) => c + c)
        .join("");
    return {
      r: parseInt(h.slice(0, 2), 16),
      g: parseInt(h.slice(2, 4), 16),
      b: parseInt(h.slice(4, 6), 16),
    };
  }

  function rgbToHsl(r, g, b) {
    r /= 255;
    g /= 255;
    b /= 255;
    const max = Math.max(r, g, b),
      min = Math.min(r, g, b);
    let h = 0,
      s = 0;
    const l = (max + min) / 2;
    if (max !== min) {
      const d = max - min;
      s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
      switch (max) {
        case r:
          h = ((g - b) / d + (g < b ? 6 : 0)) / 6;
          break;
        case g:
          h = ((b - r) / d + 2) / 6;
          break;
        case b:
          h = ((r - g) / d + 4) / 6;
          break;
      }
    }
    return {
      h: Math.round(h * 360),
      s: Math.round(s * 100),
      l: Math.round(l * 100),
    };
  }

  function parseColor(str) {
    str = str.trim();
    if (!str) return null;

    // HEX
    if (/^#[0-9a-fA-F]{3,8}$/.test(str)) {
      const { r, g, b } = hexToRgb(str);
      return rgbToHsl(r, g, b);
    }

    // RGB / RGBA
    const rgb = str.match(
      /^rgba?\s*\(\s*([\d.]+)\s*,\s*([\d.]+)\s*,\s*([\d.]+)/,
    );
    if (rgb) return rgbToHsl(+rgb[1], +rgb[2], +rgb[3]);

    // HSL / HSLA
    const hsl = str.match(
      /^hsla?\s*\(\s*([\d.]+)\s*,\s*([\d.]+)%?\s*,\s*([\d.]+)%?/,
    );
    if (hsl)
      return {
        h: Math.round(+hsl[1]),
        s: Math.round(+hsl[2]),
        l: Math.round(+hsl[3]),
      };

    return null;
  }

  let lastInitialColor = "";
  $: if (initialColor !== lastInitialColor) {
    lastInitialColor = initialColor;
    const parsed = parseColor(initialColor);
    if (parsed) {
      hue = parsed.h;
      saturation = parsed.s;
      lightness = parsed.l;
    }
  }

  let slPickerEl;
  let hueSliderEl;

  function hslToRgb(h, s, l) {
    s /= 100;
    l /= 100;
    const c = (1 - Math.abs(2 * l - 1)) * s;
    const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
    const m = l - c / 2;
    let r = 0,
      g = 0,
      b = 0;

    if (h < 60) {
      r = c;
      g = x;
      b = 0;
    } else if (h < 120) {
      r = x;
      g = c;
      b = 0;
    } else if (h < 180) {
      r = 0;
      g = c;
      b = x;
    } else if (h < 240) {
      r = 0;
      g = x;
      b = c;
    } else if (h < 300) {
      r = x;
      g = 0;
      b = c;
    } else {
      r = c;
      g = 0;
      b = x;
    }

    return {
      r: Math.round((r + m) * 255),
      g: Math.round((g + m) * 255),
      b: Math.round((b + m) * 255),
    };
  }

  function rgbToHex(r, g, b) {
    return "#" + [r, g, b].map((x) => x.toString(16).padStart(2, "0")).join("");
  }

  $: currentColor = `hsl(${hue}, ${saturation}%, ${lightness}%)`;
  $: rgb = hslToRgb(hue, saturation, lightness);
  $: hex = rgbToHex(rgb.r, rgb.g, rgb.b);

  $: cursorX = saturation; // percentage, directly maps to left%
  $: cursorY = (() => {
    const lightnessAtTop = 100 - saturation / 2;
    if (lightnessAtTop <= 0) return 100;
    return Math.max(0, Math.min(100, (1 - lightness / lightnessAtTop) * 100));
  })();

  function handleSLPickerMove(e) {
    if (!slPickerEl) return;
    const rect = slPickerEl.getBoundingClientRect();
    const xRaw = Math.max(0, Math.min(e.clientX - rect.left, rect.width));
    const yRaw = Math.max(0, Math.min(e.clientY - rect.top, rect.height));
    const xNorm = xRaw / rect.width; // 0..1
    const yNorm = yRaw / rect.height; // 0..1

    saturation = Math.round(xNorm * 100);
    const lightnessAtTop = 100 - saturation / 2;
    lightness = Math.round((1 - yNorm) * lightnessAtTop);
  }

  function handleHueSliderMove(e) {
    if (!hueSliderEl) return;
    const rect = hueSliderEl.getBoundingClientRect();
    const y = Math.max(0, Math.min(e.clientY - rect.top, rect.height));
    hue = Math.round((y / rect.height) * 360);
  }

  function handleMouseMove(e) {
    if (isDraggingSL) handleSLPickerMove(e);
    if (isDraggingHue) handleHueSliderMove(e);
  }

  function handleMouseUp() {
    isDraggingSL = false;
    isDraggingHue = false;
  }

  function copyText(text, label) {
    navigator.clipboard.writeText(text);
    addToast(`Copied ${label}`);
  }
</script>

<svelte:window on:mousemove={handleMouseMove} on:mouseup={handleMouseUp} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="color-picker">
  <div class="background-blob" style="background-color: {currentColor};"></div>
  <div class="left-section">
    <!-- Preview -->
    <div class="preview" style="background-color: {currentColor}"></div>

    <!-- Saturation/Lightness Picker -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      bind:this={slPickerEl}
      class="sl-picker"
      style="background: linear-gradient(to bottom, transparent, black), linear-gradient(to right, white, hsl({hue}, 100%, 50%))"
      on:mousedown={(e) => {
        isDraggingSL = true;
        handleSLPickerMove(e);
      }}
    >
      <div class="sl-cursor" style="left: {cursorX}%; top: {cursorY}%"></div>
    </div>
  </div>

  <!-- Hue Slider -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    bind:this={hueSliderEl}
    class="hue-slider"
    on:mousedown={(e) => {
      isDraggingHue = true;
      handleHueSliderMove(e);
    }}
  >
    <div class="hue-cursor" style="top: {(hue / 360) * 100}%"></div>
  </div>

  <!-- Right section: Color values -->
  <div class="values-section">
    <button
      class="value-box"
      on:click={() => copyText(`${hue}°, ${saturation}%, ${lightness}%`, "HSL")}
    >
      <span class="value-title">HSL</span>
      <span class="value-content mono">{hue}°, {saturation}%, {lightness}%</span>
    </button>

    <button
      class="value-box"
      on:click={() => copyText(`${rgb.r}, ${rgb.g}, ${rgb.b}`, "RGB")}
    >
      <span class="value-title">RGB</span>
      <span class="value-content mono">{rgb.r}, {rgb.g}, {rgb.b}</span>
    </button>

    <button
      class="value-box"
      on:click={() => copyText(hex, "HEX")}
    >
      <span class="value-title">HEX</span>
      <span class="value-content mono">{hex}</span>
    </button>
  </div>
</div>

<style>
  .color-picker {
    display: flex;
    gap: 20px;
    border-radius: 8px;
    margin: 3rem auto;
    align-items: stretch;
    justify-content: center;
    padding: 20px;
  }

  <!-- .background-blob { -->
  <!--   position: fixed; -->
  <!--   bottom: 0%; -->
  <!--   right: 0; -->
  <!--   transform: translate(-86%, -50%); -->
  <!--   height: 56%; -->
  <!--   width: 57%; -->
  <!--   z-index: -1; -->
  <!--   background-size: 300% 300%; -->
  <!--   opacity: 0.2; -->
  <!--   filter: blur(120px); -->
  <!--   border-radius: 50%; -->
  <!-- } -->

  .left-section {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .preview {
    width: 240px;
    height: 80px;
    border-radius: 111px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
    border: 2px solid rgba(255, 255, 255, 0.2);
  }

  .sl-picker {
    width: 240px;
    height: 240px;
    border-radius: 8px;
    cursor: crosshair;
    position: relative;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  }

  .sl-cursor {
    position: absolute;
    width: 18px;
    height: 18px;
    border: 3px solid white;
    border-radius: 50%;
    box-shadow:
      0 0 0 1px rgba(0, 0, 0, 0.3),
      0 2px 4px rgba(0, 0, 0, 0.4);
    pointer-events: none;
    transform: translate(-50%, -50%);
  }

  .hue-slider {
    width: 40px;
    border-radius: 8px;
    cursor: grab;
    position: relative;
    background: linear-gradient(
      to bottom,
      #ff0000 0%,
      #ffff00 17%,
      #00ff00 33%,
      #00ffff 50%,
      #0000ff 67%,
      #ff00ff 83%,
      #ff0000 100%
    );
  }

  .hue-cursor {
    position: absolute;
    width: 100%;
    height: 6px;
    background: white;
    border: 2px solid rgba(0, 0, 0, 0.3);
    border-radius: 2px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
    pointer-events: none;
    transform: translateY(-50%);
  }

  .values-section {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: 24px;
    min-width: 284px;
  }

  .value-box {
    color: white;
    margin-left: 24px;
    padding: 22px;
    border-radius: 22px;
    border: 1px solid rgba(0, 0, 0, 0.1);
    background-color: rgba(60, 60, 60, 0.3);
    display: flex;
    justify-content: space-between;
    align-items: center;
    transition: background-color 200ms ease, border 200ms ease, transform 100ms ease;
    position: relative;
    overflow: hidden;
  }

  .value-box:hover {
    transform: scale(1.03);
    background-color: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.3);
  }
  .value-box:active {
    transform: scale(1);
  }

  .value-content {
    font-size: 16px;
    font-weight: 500;
    font-family: 'JetBrainsMono Nerd Font', 'Fira Code', 'Cascadia Mono', monospace;
  }
</style>
