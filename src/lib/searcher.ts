// lib/searcher.ts
import { invoke } from "@tauri-apps/api/core";
import type { SearchResult } from "../stores/search";
import { resultItems, resultType } from "../stores/search";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { addToast } from "../stores/toasts";

export async function execute(action_id: string, name: string, currentQuery: string) {
  // Client-side QR code action — no Rust round-trip needed.
  if (action_id.startsWith("qr:")) {
    const text = action_id.slice(3);
    resultItems.set([{ name: text, actions: [] }]);
    resultType.set("QrCode");
    return;
  }

  try {
    const result = await invoke<string>("execute", {
      actionId: action_id,
      name,
      query: currentQuery,
    });

    if (result === "copied") {
      addToast("Copied to clipboard");
    } else if (result === "stay") {
      // action manages its own UI (e.g. show_modal) - keep window visible
    } else if (result === "error") {
      addToast("Something went wrong", "error");
    } else if (result.startsWith("toasted:")) {
      addToast(result.slice("toasted:".length));
    } else {
      await getCurrentWindow().hide();
    }
  } catch (e: any) {
    addToast(e?.message ?? "Something went wrong", "error");
    console.error("Execute error:", e);
  }
}

export async function search(query: string): Promise<SearchResult | null> {
  try {
    const result = await invoke<SearchResult | null>("search", { query });
    return result;
  } catch (e) {
    console.error("Search failed:", e);
    return {
      results: [{ name: "error", actions: [] }],
      result_type: "List",
    };
  }
}
