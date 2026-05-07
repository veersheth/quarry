import { writable } from "svelte/store";

export type Action = {
  id: string;
  name: string;
};

export type ResultItem = {
  name: string;
  actions: Action[];
  description?: string;
  icon?: string;
  thumbnail?: string;
  pinned?: boolean;
  draggable_path?: string;
};

export type ResultType = "List" | "Grid" | "WebSearch" | "Markdown" | "Clipboard" | "ColorPicker" | "Home" | "Media" | "Math" | "Camera" | "Ai";

export type SearchResult = {
  results: ResultItem[];
  result_type: ResultType;
  searcher?: string;
};

export type ContextMenuState = {
  open: boolean;
  item: ResultItem | null;
  x: number;
  y: number;
  activeIndex: number;
  searchQuery: string;
};

export const query = writable("");
export const resultItems = writable<ResultItem[]>([]);
export const resultType = writable<ResultType>("List");
export const activeIndex = writable(0);
export const aiSubmitQuery = writable<string>("");
export const searcherName = writable<string>("");

export const contextMenu = writable<ContextMenuState>({
  open: false,
  item: null,
  x: 0,
  y: 0,
  activeIndex: 0,
  searchQuery: "",
});

export function openContextMenu(item: ResultItem, x: number, y: number) {
  contextMenu.set({ open: true, item, x, y, activeIndex: 0, searchQuery: "" });
}

export function closeContextMenu() {
  contextMenu.update(s => ({ ...s, open: false, item: null, searchQuery: "" }));
}
