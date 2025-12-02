import { invoke } from "@tauri-apps/api/core";
import type { SearchResult } from "../stores/search";
import { getCurrentWindow } from "@tauri-apps/api/window";

export async function execute(action_id: string, name: string, currentQuery: string) {
  console.log(action_id);
  try {
    await invoke("execute", { 
      actionId: action_id,
      name,
      query: currentQuery 
    });
    await getCurrentWindow().hide();
  } catch (e) {
    console.error("Execute error:", e);
  }
}

export async function search(query: string): Promise<SearchResult> {
  try {
    const result: SearchResult = await invoke("search", { query });
    return result;
  } catch (e) {
    console.error("Search failed:", e);
    return {
      results: [{ name: "error", action_id: "notify-send 'Error'" }],
      result_type: "List",
    };
  }
}
