import { invoke } from "@tauri-apps/api/core";
import type { ResultItem, ResultType, SearchResult } from "../stores/search";

const searchCache = new Map<string, SearchResult>();
const cacheTimestamps = new Map<string, number>();
const CACHE_TTL = 5 * 60 * 1000;

export async function execute(executable: string) {
  try {
    await invoke("execute", { executable });
  } catch (e) {
    console.error("Execute error:", e);
  }
}

export async function search(query: string): Promise<SearchResult> {
  const cached = searchCache.get(query);
  const timestamp = cacheTimestamps.get(query);

  if (cached && timestamp && Date.now() - timestamp < CACHE_TTL) {
    return cached;
  }

  try {
    const result: SearchResult = await invoke("search", { query });
    searchCache.set(query, result);
    cacheTimestamps.set(query, Date.now());
    return result;
  } catch (e) {
    console.error("Search failed:", e);
    return {
      results: [{ name: "error", exec: "notify-send 'Error'" }],
      result_type: "List",
    };
  }
}

