pub mod home;
pub mod time;

// ── Shared icon paths ─────────────────────────────────────────────────────────
// Single source of truth for every icon path used across searchers.
// All paths are relative to the webview root (static/ in the source tree).
pub const ICON_FOLDER:      &str = "icons/folder.png";
pub const ICON_FILE:        &str = "icons/file.png";
pub const ICON_BOOKMARK:    &str = "icons/bookmark.png";
pub const ICON_SETTINGS:    &str = "icons/settings.png";
pub const ICON_MATH:        &str = "icons/math.png";
pub const ICON_TRANSPARENT: &str = "icons/icon-transparent.png";

/// Data URI for the script run icon (green play triangle).
/// Canonical SVG is at static/icons/script.svg; base64 here avoids WebKitGTK
/// failing to render SVGs loaded via <img> file paths.
pub const SCRIPT_ICON: &str = "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCI+PHBhdGggZD0iTTggNiBMMTkgMTIgTDggMTggWiIgZmlsbD0iIzIyYzU1ZSIgc3Ryb2tlPSIjMjJjNTVlIiBzdHJva2Utd2lkdGg9IjIiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIvPjwvc3ZnPg==";
pub mod ai;
pub mod currency;
pub mod default;
pub mod apps;
pub mod emojis;
pub mod lorem;
pub mod math;
pub mod shell;
pub mod dictionary;
pub mod system;
pub mod web_searchers;
pub mod clipboard;
pub mod colorpicker;
pub mod files;
pub mod bookmarks;
pub mod camera;
pub mod note;
pub mod timer;
pub mod screenshots;
pub mod shortcuts;
pub mod scripts;
pub mod qrcode;

use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use tauri::AppHandle;
use crate::types::{ResultItem, SearchResult};

pub trait SearchProvider {
    fn search(&self, query: &str, app: &AppHandle) -> SearchResult;
    fn name(&self) -> String { String::new() }

    fn fuzzy_filter(&self, items: Vec<ResultItem>, query: &str) -> Vec<ResultItem> {
        if query.is_empty() {
            return items;
        }
        let matcher = SkimMatcherV2::default().ignore_case();
        let mut scored: Vec<(ResultItem, i64)> = items
            .into_iter()
            .filter_map(|item| {
                let score = fuzzy_score(&matcher, &item, query);
                if score > 0 { Some((item, score)) } else { None }
            })
            .collect();
        scored.sort_unstable_by(|a, b| b.1.cmp(&a.1));
        scored.into_iter().map(|(item, _)| item).collect()
    }
}

fn fuzzy_score(matcher: &SkimMatcherV2, item: &ResultItem, query: &str) -> i64 {
    let q = query.to_lowercase();
    let name = item.name.to_lowercase();
    let desc = item.description.as_deref().unwrap_or("").to_lowercase();
    let combined = format!("{} {}", name, desc);
    let combined_score = matcher.fuzzy_match(&combined, &q).unwrap_or(0);
    let name_score = matcher.fuzzy_match(&name, &q)
        .map(|s| s * 2)
        .unwrap_or(0);
    combined_score.max(name_score)
}
