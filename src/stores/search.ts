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
  thumbnail_key?: string;
  pinned?: boolean;
  group?: string;
  draggable_path?: string;
  ocr_text?: string;
};

export type ResultType = "List" | "Grid" | "Apps" | "WebSearch" | "Markdown" | "Clipboard" | "ColorPicker" | "Home" | "Media" | "Math" | "Camera" | "Ai" | "Screenshots" | "QrCode" | "Fonts" | "Files";

export type SearchResult = {
  results: ResultItem[];
  result_type: ResultType;
  searcher?: string;
  seq?: number;
};

export type ContextMenuState = {
  open: boolean;
  item: ResultItem | null;
  activeIndex: number;
  searchQuery: string;
  onSelect: ((actionId: string) => void | Promise<void>) | null;
};

export const mouseHasMoved = writable(false);

export type ModalButton = { label: string; kind?: string; action_id?: string };
export type ModalStoreState = {
  open: boolean;
  body: string;
  buttons: ModalButton[];
  activeIndex: number;
};
export const modalStore = writable<ModalStoreState>({
  open: false, body: "", buttons: [], activeIndex: 0,
});
export function openModal(body: string, buttons: ModalButton[] = [{ label: "Dismiss" }]) {
  modalStore.set({ open: true, body, buttons, activeIndex: 0 });
}
export function closeModal() {
  modalStore.update(s => ({ ...s, open: false }));
}

export const bookmarkCreatorOpen = writable(false);
export const bookmarkBeingEdited = writable<{ name: string; url: string } | null>(null);

export const query = writable("");
export const resultItems = writable<ResultItem[]>([]);
export const resultType = writable<ResultType>("List");
export const activeIndex = writable(0);
export const aiSubmitQuery = writable<string>("");
export const searcherName = writable<string>("");

export const contextMenu = writable<ContextMenuState>({
  open: false,
  item: null,
  activeIndex: 0,
  searchQuery: "",
  onSelect: null,
});

export const aiContextMenuOpener = writable<(() => void) | null>(null);

export function openContextMenu(
  item: ResultItem,
  onSelect: ContextMenuState["onSelect"] = null,
) {
  contextMenu.set({ open: true, item, activeIndex: 0, searchQuery: "", onSelect });
}

export function closeContextMenu() {
  contextMenu.update(s => ({ ...s, open: false, item: null, searchQuery: "", onSelect: null }));
}
