import type { Writable } from "svelte/store";
import type { ResultItem } from "../stores/search";
import { execute } from "./searcher";
import { get } from "svelte/store";
import { query } from "../stores/search";

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

  //////////////////////////////////////////////////////////////
  /////////////////    navigation    ///////////////////////////
  //////////////////////////////////////////////////////////////

  let items: ResultItem[];
  resultItems.subscribe(v => items = v)();

  if (!items || items.length === 0) return;

  activeIndex.update((index) => {
    if (event.key === "Tab" && !event.shiftKey) return (index + 1) % items.length;
    if (event.key === "Tab" && event.shiftKey) return index === 0 ? items.length - 1 : index - 1;
    if (event.key === "n" && event.ctrlKey) return (index + 1) % items.length;
    if (event.key === "p" && event.ctrlKey) return index === 0 ? items.length - 1 : index - 1;
    if (event.key === "Enter") {
      const currentQuery = get(query);
      execute(items[index].action_id, items[index].name, currentQuery);
    }
    return index;
  });
}

