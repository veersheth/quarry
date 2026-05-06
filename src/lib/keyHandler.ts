import type { Writable } from "svelte/store";
import type { ResultItem } from "../stores/search";
import { execute } from "./searcher";
import { get } from "svelte/store";
import { tick } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import { query, resultType, aiSubmitQuery, contextMenu, closeContextMenu } from "../stores/search";

function deleteWordFromEnd(str: string): string {
  let pos = str.length;
  while (pos > 0 && /\s/.test(str[pos - 1])) pos--;
  if (pos > 0) {
    const charClass = /\w/.test(str[pos - 1]) ? "word" : "punct";
    if (charClass === "word") {
      while (pos > 0 && /\w/.test(str[pos - 1])) pos--;
    } else {
      while (pos > 0 && /[^\w\s]/.test(str[pos - 1])) pos--;
    }
  }
  return str.slice(0, pos);
}

export function handleKeydown(
  event: KeyboardEvent,
  searchInput: HTMLInputElement,
  activeIndex: Writable<number>,
  resultItems: Writable<ResultItem[]>,
  appWindow: any
) {
  const isAiMode = get(resultType) === "Ai";
  const menu = get(contextMenu);

  // --- Context menu is open: intercept all keys ---
  if (menu.open && menu.item) {
    const searchQ = menu.searchQuery ?? "";
    const filtered = searchQ
      ? menu.item.actions.filter(a => a.name.toLowerCase().includes(searchQ.toLowerCase()))
      : menu.item.actions;
    const count = filtered.length;

    if (event.key === "Escape") {
      event.preventDefault();
      closeContextMenu();
      return;
    }

    if (event.key === "Backspace") {
      event.preventDefault();
      contextMenu.update(s => ({ ...s, searchQuery: s.searchQuery.slice(0, -1), activeIndex: 0 }));
      return;
    }

    if (event.key === "w" && event.ctrlKey) {
      event.preventDefault();
      contextMenu.update(s => ({ ...s, searchQuery: deleteWordFromEnd(s.searchQuery), activeIndex: 0 }));
      return;
    }

    if (event.key === "ArrowDown" || (event.key === "n" && event.ctrlKey)) {
      event.preventDefault();
      if (count > 0) contextMenu.update(s => ({ ...s, activeIndex: (s.activeIndex + 1) % count }));
      return;
    }

    if (event.key === "ArrowUp" || (event.key === "p" && event.ctrlKey)) {
      event.preventDefault();
      if (count > 0) contextMenu.update(s => ({ ...s, activeIndex: s.activeIndex === 0 ? count - 1 : s.activeIndex - 1 }));
      return;
    }

    if (event.key === "Enter") {
      event.preventDefault();
      if (filtered[menu.activeIndex]) {
        execute(filtered[menu.activeIndex].id, menu.item!.name, get(query));
      }
      closeContextMenu();
      return;
    }

    // Alt+1-4: directly execute that action
    if (event.altKey && ["1","2","3","4"].includes(event.key)) {
      event.preventDefault();
      const n = parseInt(event.key) - 1;
      if (filtered[n]) {
        execute(filtered[n].id, menu.item!.name, get(query));
        closeContextMenu();
      }
      return;
    }

    // Printable character: append to search query
    if (event.key.length === 1 && !event.ctrlKey && !event.altKey && !event.metaKey) {
      event.preventDefault();
      contextMenu.update(s => ({ ...s, searchQuery: s.searchQuery + event.key, activeIndex: 0 }));
      return;
    }

    return;
  }

  if (!isAiMode && searchInput && document.activeElement !== searchInput) {
    searchInput.focus();
  }

  if (event.key === "Escape") {
    event.preventDefault();
    appWindow?.hide();
    return;
  }

  if (event.key === "," && event.ctrlKey) {
    event.preventDefault();
    invoke("exec_func", { name: "reload_quarry", params: [] as string[] }).catch(console.error);
    return;
  }

  if (isAiMode) {
    if (event.key === "Enter") {
      event.preventDefault();
      const q = get(query).replace(/^ai\s+/i, "").trim();
      if (q) aiSubmitQuery.set(q);
    }
  }

  if (event.key === "w" && event.ctrlKey) {
    event.preventDefault();
    const input = event.target as HTMLInputElement;
    const current = get(query);
    const cursorPos = input.selectionStart ?? current.length;
    if (cursorPos === 0) return;

    const before = current.slice(0, cursorPos);
    const after = current.slice(cursorPos);
    const newBefore = deleteWordFromEnd(before);

    query.set(newBefore + after);
    requestAnimationFrame(() => {
      input.selectionStart = input.selectionEnd = newBefore.length;
    });
    return;
  }

  // open context menu for active item with ctrl k
  if (event.key === "k" && event.ctrlKey) {
    event.preventDefault();
    const items = get(resultItems);
    const idx = get(activeIndex);
    const item = items[idx];
    if (item && item.actions.length > 1) {
      contextMenu.set({ open: true, item, x: 0, y: 0, activeIndex: 0, searchQuery: "" });
      window.dispatchEvent(new CustomEvent("open-context-menu-at-active"));
    }
    return;
  }

  const items = get(resultItems);
  if (!items || items.length === 0) return;

  if (["ArrowDown", "ArrowUp", "Tab", "Enter"].includes(event.key) ||
    (event.key === "n" && event.ctrlKey) ||
    (event.key === "p" && event.ctrlKey) ||
    (event.altKey && ["1", "2", "3", "4"].includes(event.key))) {
    event.preventDefault();
  }

  activeIndex.update((index) => {
    if (event.key === "ArrowDown") return Math.min(index + 1, items.length - 1);
    if (event.key === "ArrowUp") return Math.max(index - 1, 0);
    if (event.key === "Tab" && !event.shiftKey) return Math.min(index + 1, items.length - 1);
    if (event.key === "Tab" && event.shiftKey) return Math.max(index - 1, 0);
    if (event.key === "n" && event.ctrlKey) return Math.min(index + 1, items.length - 1);
    if (event.key === "p" && event.ctrlKey) return Math.max(index - 1, 0);
    if (event.key === "Enter") { runItemAction(items[index]); return index; }
    if (event.key === "1" && event.altKey) return items.length > 0 ? 0 : index;
    if (event.key === "2" && event.altKey) return items.length > 1 ? 1 : index;
    if (event.key === "3" && event.altKey) return items.length > 2 ? 2 : index;
    if (event.key === "4" && event.altKey) return items.length > 3 ? 3 : index;
    return index;
  });

  tick().then(() => {
    document.querySelector('[data-active="true"]')?.scrollIntoView({ block: "nearest" });
  });
}

export function runItemAction(item: ResultItem) {
  execute(item.actions[0]?.id, item.name, get(query));
}
