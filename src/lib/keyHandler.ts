import type { Writable } from "svelte/store";
import type { ResultItem } from "../stores/search";
import { execute } from "./searcher";
import { get } from "svelte/store";
import { query, resultType, aiSubmitQuery } from "../stores/search";

export function handleKeydown(
  event: KeyboardEvent,
  searchInput: HTMLInputElement,
  activeIndex: Writable<number>,
  resultItems: Writable<ResultItem[]>,
  appWindow: any
) {
  const isAiMode = get(resultType) === "Ai";

  // Don't steal focus back to search bar when AI is showing
  if (!isAiMode && searchInput && document.activeElement !== searchInput) {
    searchInput.focus();
  }

  if (event.key === "Escape") {
    event.preventDefault();
    appWindow?.hide();
    return;
  }

  if (isAiMode) {
    // Enter submits the query to AI
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
    const cursorPos = input.selectionStart ?? 0;
    if (cursorPos === 0) return;

    const before = current.slice(0, cursorPos);
    const after = current.slice(cursorPos);
    let pos = before.length;

    while (pos > 0 && /\s/.test(before[pos - 1])) pos--;
    if (pos > 0) {
      const charClass = /\w/.test(before[pos - 1]) ? "word" : "punct";
      if (charClass === "word") {
        while (pos > 0 && /\w/.test(before[pos - 1])) pos--;
      } else {
        while (pos > 0 && /[^\w\s]/.test(before[pos - 1])) pos--;
      }
    }

    query.set(before.slice(0, pos) + after);
    requestAnimationFrame(() => {
      input.selectionStart = input.selectionEnd = pos;
    });
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
    if (event.key === "ArrowDown") return (index + 1) % items.length;
    if (event.key === "ArrowUp") return index === 0 ? items.length - 1 : index - 1;
    if (event.key === "Tab" && !event.shiftKey) return (index + 1) % items.length;
    if (event.key === "Tab" && event.shiftKey) return index === 0 ? items.length - 1 : index - 1;
    if (event.key === "n" && event.ctrlKey) return (index + 1) % items.length;
    if (event.key === "p" && event.ctrlKey) return index === 0 ? items.length - 1 : index - 1;
    if (event.key === "Enter") { runItemAction(items[index]); return index; }
    if (event.key === "1" && event.altKey && items[0]) { runItemAction(items[0]); return index; }
    if (event.key === "2" && event.altKey && items[1]) { runItemAction(items[1]); return index; }
    if (event.key === "3" && event.altKey && items[2]) { runItemAction(items[2]); return index; }
    if (event.key === "4" && event.altKey && items[3]) { runItemAction(items[3]); return index; }
    return index;
  });
}

export function runItemAction(item: ResultItem) {
  execute(item.action_id, item.name, get(query));
}
