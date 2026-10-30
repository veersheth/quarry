pub mod home;
pub mod time;

// ── Shared icon paths ─────────────────────────────────────────────────────────
// Single source of truth for every icon path used across searchers.
// All paths are relative to the webview root (static/ in the source tree).
pub const ICON_FOLDER: &str = "icons/folder.png";
pub const ICON_FILE: &str = "icons/file.png";
pub const ICON_BOOKMARK: &str = "icons/bookmark.png";
pub const ICON_SETTINGS: &str = "icons/settings.png";
pub const ICON_MATH: &str = "icons/math.png";
pub const ICON_TRANSPARENT: &str = "icons/icon-transparent.png";

/// Data URI for the script run icon (green play triangle).
/// Canonical SVG is at static/icons/script.svg; base64 here avoids WebKitGTK
/// failing to render SVGs loaded via <img> file paths.
pub const SCRIPT_ICON: &str = "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCI+PHBhdGggZD0iTTggNiBMMTkgMTIgTDggMTggWiIgZmlsbD0iIzIyYzU1ZSIgc3Ryb2tlPSIjMjJjNTVlIiBzdHJva2Utd2lkdGg9IjIiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIvPjwvc3ZnPg==";
pub mod ai;
pub mod apps;
pub mod bookmarks;
pub mod camera;
pub mod clipboard;
pub mod colorpicker;
pub mod currency;
pub mod default;
pub mod dictionary;
pub mod emojis;
pub mod files;
pub mod fonts;
pub mod lorem;
pub mod math;
pub mod meta;
pub mod qrcode;
pub mod screenshots;
pub mod scripts;
pub mod shell;
pub mod shortcuts;
pub mod system;
pub mod timer;
pub mod web_searchers;

use crate::types::{ResultItem, SearchResult};
use tauri::AppHandle;

pub trait SearchProvider {
    fn search(&self, query: &str, app: &AppHandle) -> SearchResult;
    fn name(&self) -> String {
        String::new()
    }

    fn fuzzy_filter(&self, items: Vec<ResultItem>, query: &str) -> Vec<ResultItem> {
        if query.is_empty() {
            return items;
        }
        let mut scored: Vec<(ResultItem, i64)> = items
            .into_iter()
            .filter_map(|item| {
                let score = fuzzy_score(&item, query);
                if score > 0 {
                    Some((item, score))
                } else {
                    None
                }
            })
            .collect();
        scored.sort_unstable_by_key(|item| std::cmp::Reverse(item.1));
        scored.into_iter().map(|(item, _)| item).collect()
    }
}

fn fuzzy_score(item: &ResultItem, query: &str) -> i64 {
    let name_score = crate::search_utils::smart_match(&item.name, query)
        .map(|score| score * 2)
        .unwrap_or(0);
    let description_score = item
        .description
        .as_deref()
        .and_then(|description| crate::search_utils::smart_match(description, query))
        .unwrap_or(0);
    name_score.max(description_score)
}
