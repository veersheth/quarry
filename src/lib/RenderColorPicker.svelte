<script>
  let hue = 0;
  let saturation = 100;
  let lightness = 50;
  let isDraggingSL = false;
  let isDraggingHue = false;
  
  let slPickerEl;
  let hueSliderEl;

  function hslToRgb(h, s, l) {
    s /= 100;
    l /= 100;
    const c = (1 - Math.abs(2 * l - 1)) * s;
    const x = c * (1 - Math.abs((h / 60) % 2 - 1));
    const m = l - c / 2;
    let r = 0, g = 0, b = 0;
    
    if (h < 60) { r = c; g = x; b = 0; }
    else if (h < 120) { r = x; g = c; b = 0; }
    else if (h < 180) { r = 0; g = c; b = x; }
    else if (h < 240) { r = 0; g = x; b = c; }
    else if (h < 300) { r = x; g = 0; b = c; }
    else { r = c; g = 0; b = x; }
    
    return {
      r: Math.round((r + m) * 255),
      g: Math.round((g + m) * 255),
      b: Math.round((b + m) * 255)
    };
  }

  function rgbToHex(r, g, b) {
    return '#' + [r, g, b].map(x => x.toString(16).padStart(2, '0')).join('');
  }

  $: currentColor = `hsl(${hue}, ${saturation}%, ${lightness}%)`;
  $: rgb = hslToRgb(hue, saturation, lightness);
  $: hex = rgbToHex(rgb.r, rgb.g, rgb.b);

  function handleSLPickerMove(e) {
    if (!slPickerEl) return;
    const rect = slPickerEl.getBoundingClientRect();
    const x = Math.max(0, Math.min(e.clientX - rect.left, rect.width));
    const y = Math.max(0, Math.min(e.clientY - rect.top, rect.height));
    saturation = Math.round((x / rect.width) * 100);
    lightness = Math.round(100 - (y / rect.height) * 100);
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
</script>

<svelte:window 
  on:mousemove={handleMouseMove}
  on:mouseup={handleMouseUp}
/>

<div class="color-picker">
  <div class="left-section">
    <!-- Preview -->
    <div class="preview" style="background-color: {currentColor}"></div>
    
    <!-- Saturation/Lightness Picker -->
    <div
      bind:this={slPickerEl}
      class="sl-picker"
      style="background: linear-gradient(to top, black, transparent), linear-gradient(to right, white, hsl({hue}, 100%, 50%))"
      on:mousedown={(e) => {
        isDraggingSL = true;
        handleSLPickerMove(e);
      }}
    >
      <div
        class="sl-cursor"
        style="left: {saturation}%; top: {100 - lightness}%"
      ></div>
    </div>
  </div>

  <!-- Hue Slider -->
  <div
    bind:this={hueSliderEl}
    class="hue-slider"
    on:mousedown={(e) => {
      isDraggingHue = true;
      handleHueSliderMove(e);
    }}
  >
    <div
      class="hue-cursor"
      style="top: {(hue / 360) * 100}%"
    ></div>
  </div>

  <!-- Right section: Color values -->
  <div class="values-section">
    <!-- HSL -->
    <div class="value-box">
      <span class="value-title">HSL</span>
      <span class="value-content mono">{hue}°, {saturation}%, {lightness}%</span>
    </div>

    <!-- RGB -->
    <div class="value-box">
      <span class="value-title">RGB</span>
      <span class="value-content mono">{rgb.r}, {rgb.g}, {rgb.b}</span>
    </div>

    <!-- HEX -->
    <div class="value-box">
      <span class="value-title">HEX</span>
      <span class="value-content mono">{hex}</span>
    </div>
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

  .left-section {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .preview {
    width: 240px;
    height: 80px;
    border-radius: 8px;
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
    box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.3), 0 2px 4px rgba(0, 0, 0, 0.4);
    pointer-events: none;
    transform: translate(-50%, -50%);
  }

  .hue-slider {
    width: 40px;
    border-radius: 8px;
    cursor: grab;
    position: relative;
    background: linear-gradient(to bottom, 
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
    cursor: pointer;
    padding: 22px;
    border-radius: 8px;
    border: 1px solid rgba(0, 0, 0, 0.1);
    background-color: rgba(60, 60, 60, 0.3);
    display: flex;
    justify-content: space-between;
  }

  .value-content {
    font-size: 16px;
    font-weight: 500;
    font-family: 'JetBrainsMono Nerd Font', 'Courier New', monospace;
  }
</style>
