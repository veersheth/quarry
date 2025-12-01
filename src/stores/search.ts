import { writable } from "svelte/store";

export type ResultItem = {
  name: string;
  exec: string;
  description?: string;
  icon?: string;
};

export type ResultType = "List" | "Grid";

export type SearchResult = {
  results: ResultItem[];
  result_type: ResultType;
};

export const query = writable("");
export const resultItems = writable<ResultItem[]>([]);
export const resultType = writable<ResultType>("List");
export const activeIndex = writable(0);

