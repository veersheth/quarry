import type { Writable } from "svelte/store";
import type { ResultItem } from "../stores/search";
import { execute } from "./searcher";
import { get } from "svelte/store";
import { query } from "../stores/search";

// existing keyboard handler
export function handleKeydown(
  event: KeyboardEvent,
  searchInput: HTMLInputElement,
  activeIndex: Writable<number>,
  resultItems: Writable<ResultItem[]>,
  appWindow: any
) {
  if (searchInput && document.activeElement !== searchInput) {
    searchInput.focus();
  }

  if (event.key === "Escape") {
    event.preventDefault();
    appWindow?.hide();
    return;
  }

  if (event.key === "w" && event.ctrlKey) {
    event.preventDefault();
    const current = get(query);
    query.set(current.replace(/\s*\S*$/, ""));
    return;
  }

  // navigation
  let items: ResultItem[];
  resultItems.subscribe(v => items = v)();

  if (!items || items.length === 0) return;

  activeIndex.update((index) => {
    if (event.key === "ArrowDown") return (index + 1) % items.length;
    if (event.key === "ArrowUp") return index === 0 ? items.length - 1 : index - 1;
    if (event.key === "Tab" && !event.shiftKey) return (index + 1) % items.length;
    if (event.key === "Tab" && event.shiftKey) return index === 0 ? items.length - 1 : index - 1;
    if (event.key === "n" && event.ctrlKey) return (index + 1) % items.length;
    if (event.key === "p" && event.ctrlKey) return index === 0 ? items.length - 1 : index - 1;
    if (event.key === "Enter") {
      runItemAction(items[index]);
    }
    if (event.key === "1" && event.altKey) {
      runItemAction(items[0]);
    }
    if (event.key === "2" && event.altKey) {
      runItemAction(items[1]);
    }
    if (event.key === "3" && event.altKey) {
      runItemAction(items[2]);
    }
    if (event.key === "4" && event.altKey) {
      runItemAction(items[3]);
    }
    return index;
  });
}

export function runItemAction(item: ResultItem) {
  const currentQuery = get(query);
  execute(item.action_id, item.name, currentQuery);
}

