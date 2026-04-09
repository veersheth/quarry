<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { invoke } from "@tauri-apps/api/core";

  let textarea: HTMLTextAreaElement;
  let saveTimeout: ReturnType<typeof setTimeout>;
  let appWindow: ReturnType<typeof getCurrentWindow>;

  interface Theme {
    background_color: string;
    background_opacity: number;
    font_size: number;
    font_color: string;
    border_radius: number;
    border_color: string;
    border_thickness: number;
  }

  function applyTheme(t: Theme) {
    const root = document.documentElement.style;
    root.setProperty("--q-bg-color", t.background_color);
    root.setProperty("--q-bg-opacity", String(t.background_opacity));
    root.setProperty("--q-font-size", `${t.font_size}px`);
    root.setProperty("--q-font-color", t.font_color);
    root.setProperty("--q-border-color", t.border_color);
    root.setProperty("--q-border-thickness", `${t.border_thickness}px`);
  }

  const INDENT = "  "; // indent level of two spaces
  const BULLET_REGEX = /^(\s*)([-*•])\s/; 

  function getLineInfo(val: string, pos: number) {
    const lineStart = val.lastIndexOf("\n", pos - 1) + 1;
    const lineEnd = val.indexOf("\n", pos);
    const line = val.substring(lineStart, lineEnd === -1 ? val.length : lineEnd);
    const match = line.match(BULLET_REGEX);
    return {
      lineStart,
      lineEnd: lineEnd === -1 ? val.length : lineEnd,
      line,
      indent: match?.[1] ?? "",
      bullet: match?.[2] ?? null,
      content: match ? line.slice(match[0].length) : line,
      prefix: match ? match[0] : null,
    };
  }

  /** Write a new value into the uncontrolled textarea and set cursor */
  function applyEdit(newVal: string, newCursor: number) {
    textarea.value = newVal;
    textarea.selectionStart = textarea.selectionEnd = newCursor;
    scheduleSave(newVal);
  }

  function scheduleSave(value: string) {
    clearTimeout(saveTimeout);
    saveTimeout = setTimeout(() => {
      invoke("write_note", { content: value });
    }, 400);
  }

  function handleInput() {
    scheduleSave(textarea.value);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      appWindow.hide();
      return;
    }

    if (e.key === "Enter")   { handleEnter(e);        return; }
    if (e.key === "Tab")     { handleTab(e);           return; }
    if (e.key === "Backspace") { handleBackspace(e);   return; }
  }

  function handleEnter(e: KeyboardEvent) {
    const el = textarea;
    const pos = el.selectionStart;
    const val = el.value;
    const info = getLineInfo(val, pos);

    if (!info.bullet) return; 

    e.preventDefault();

    if (info.content.trim() === "") {
      // Empty bullet → outdent one level, or exit list entirely
      if (info.indent.length >= INDENT.length) {
        // Outdent: remove one indent level, keep bullet
        const newIndent = info.indent.slice(INDENT.length);
        const newLine = `${newIndent}${info.bullet} `;

        const newVal =
          val.substring(0, info.lineStart) +
          newLine +
          val.substring(info.lineEnd);

        applyEdit(newVal, info.lineStart + newLine.length);
      } else {
        const newVal = val.substring(0, info.lineStart) + val.substring(info.lineEnd);
        applyEdit(newVal, info.lineStart);
      }
      return;
    }

    const insertion = `\n${info.indent}${info.bullet} `;
    const newVal = val.substring(0, pos) + insertion + val.substring(el.selectionEnd);

    applyEdit(newVal, pos + insertion.length);
  }

  function handleTab(e: KeyboardEvent) {
    const el = textarea;
    const pos = el.selectionStart;
    const val = el.value;
    const info = getLineInfo(val, pos);

    e.preventDefault();

    if (!info.bullet) {
      const newVal = val.substring(0, pos) + INDENT + val.substring(pos);
      applyEdit(newVal, pos + INDENT.length);
      return;
    }

    if (e.shiftKey) {
      if (info.indent.length < INDENT.length) return;
      const newIndent = info.indent.slice(INDENT.length);
      const newVal = val.substring(0, info.lineStart) + newIndent + val.substring(info.lineStart + info.indent.length);
      const cursorDelta = info.indent.length - newIndent.length;
      applyEdit(newVal, Math.max(info.lineStart, pos - cursorDelta));
    } else {
      const newVal = val.substring(0, info.lineStart) + INDENT + val.substring(info.lineStart);
      applyEdit(newVal, pos + INDENT.length);
    }
  }

  function handleBackspace(e: KeyboardEvent) {
    const el = textarea;
    const pos = el.selectionStart;
    const val = el.value;

    if (el.selectionEnd !== pos) return;

    const info = getLineInfo(val, pos);
    if (!info.bullet || !info.prefix) return;

    const bulletEnd = info.lineStart + info.prefix.length;

    if (pos > bulletEnd) return;
    if (pos <= info.lineStart) return;

    e.preventDefault();

    const newVal =
      val.substring(0, info.lineStart) +
      info.content +
      val.substring(info.lineEnd);
    applyEdit(newVal, info.lineStart);
  }

  // lifecycle
  onMount(async () => {
    appWindow = getCurrentWindow();
    const theme = await invoke<Theme>("get_theme");

    applyTheme(theme);

    const saved = await invoke<string>("read_note");

    if (textarea) {
      textarea.value = saved;
      textarea.focus();
    }
  });

  onDestroy(() => clearTimeout(saveTimeout));
</script>

<svelte:window on:keydown={handleKeydown} />

<main class="container">
  <textarea
    bind:this={textarea}
    on:input={handleInput}
    placeholder="Note note, note..."
    spellcheck="false"
    autofocus
  ></textarea>
  <div class="titlebar" data-tauri-drag-region>
    <span class="title">QUARRY NOTEPAD</span>
  </div>
</main>

<style>
  :global(html, body) {
    margin: 0; padding: 0;
    height: 100%; overflow: hidden;
    background: transparent;
  }

  .container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background-color: #1a1a1f;
    overflow: hidden;
    box-sizing: border-box;
  }

  .titlebar {
    display: flex;
    align-items: center;
    height: 36px;
    padding: 0 14px;
    flex-shrink: 0;
    border-top: 1px solid rgba(255, 255, 255, 0.05);
    background: rgba(255, 255, 255, 0.02);
    user-select: none;
    cursor: grab;
  }

  .title {
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.08em;
    color: rgba(255, 255, 255, 0.2);
    user-select: none;
  }

  textarea {
    flex: 1;
    width: 100%;
    box-sizing: border-box;
    background: none;
    border: none;
    outline: none;
    resize: none;
    padding: 14px 16px 16px 16px;
    color: var(--q-font-color, rgba(255, 255, 255, 0.88));
    font-size: var(--q-font-size, 13px);
    font-family: "JetBrainsMono Nerd Font", "Fira Code", "Cascadia Code", monospace;
    line-height: 1.7;
    caret-color: #7c9ef8;
  }

  textarea::placeholder { color: rgba(255, 255, 255, 0.12); }
</style>
