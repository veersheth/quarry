pub mod time;
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
pub mod settings;
pub mod timer;
pub mod screenshots;
pub mod shortcuts;
pub mod scripts;

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
